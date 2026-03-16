use axum::{
    body::Bytes,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::Config;
use crate::security::KeychainManager;
use crate::vpn::VpnAutomator;

fn is_ip_in_subnet(ip: &str, subnet: &str) -> bool {
    // Parse subnet like "192.168.0.0/24"
    let parts: Vec<&str> = subnet.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    let network_ip = parts[0];
    let prefix_len: u32 = match parts[1].parse() {
        Ok(len) => len,
        Err(_) => return false,
    };

    // Convert IPs to u32 for comparison
    let ip_num = match ip_to_u32(ip) {
        Some(num) => num,
        None => return false,
    };

    let network_num = match ip_to_u32(network_ip) {
        Some(num) => num,
        None => return false,
    };

    // Create subnet mask
    let mask = if prefix_len == 0 {
        0
    } else {
        !0u32 << (32 - prefix_len)
    };

    // Check if IP is in subnet
    (ip_num & mask) == (network_num & mask)
}

fn ip_to_u32(ip: &str) -> Option<u32> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return None;
    }

    let mut result = 0u32;
    for (i, part) in parts.iter().enumerate() {
        let octet: u32 = part.parse().ok()?;
        result |= octet << (24 - i * 8);
    }
    Some(result)
}

#[derive(Deserialize)]
pub struct TokenRequest {
    code: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    status: String,
    message: String,
}

pub struct ServerState {
    pub config: Config,
    pub keychain: KeychainManager,
    pub vpn_automator: std::sync::Arc<VpnAutomator>,
}

pub async fn start_server(
    config: Config,
    keychain: KeychainManager,
    vpn_automator: VpnAutomator,
) -> anyhow::Result<()> {
    let port = config.server.port;
    let state = Arc::new(ServerState {
        config,
        keychain,
        vpn_automator: std::sync::Arc::new(vpn_automator),
    });

    let app = Router::new()
        .route("/token", post(handle_token))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    log::info!("Server listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn handle_token(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let client_ip = addr.ip().to_string();
    log::info!("Received token request from IP: {}", client_ip);
    log::info!("Request headers: {:?}", headers);
    log::info!("Raw request body: {:?}", body);

    // Parse JSON manually for better error handling
    let body_str = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Invalid UTF-8 in request body: {}", e);
            let response = TokenResponse {
                status: "error".to_string(),
                message: "Invalid UTF-8 in request body".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    };

    let payload: TokenRequest = match serde_json::from_str(body_str) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to parse JSON body: {} (body: '{}')", e, body_str);
            let response = TokenResponse {
                status: "error".to_string(),
                message: format!("Failed to parse the request body as JSON: {}", e),
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    };

    // Check IP whitelist (subnet-based)
    if !is_ip_in_subnet(&client_ip, &state.config.server.allowed_subnet) {
        log::warn!(
            "Unauthorized IP: {} (allowed subnet: {})",
            client_ip,
            state.config.server.allowed_subnet
        );
        let response = TokenResponse {
            status: "error".to_string(),
            message: "Unauthorized IP address".to_string(),
        };
        return (StatusCode::FORBIDDEN, Json(response));
    }

    // Check auth token
    let auth_token = match headers.get("x-auth-token") {
        Some(token) => token.to_str().unwrap_or(""),
        None => "",
    };

    let expected_token = match crate::security::KeychainManager::get_password("auth_token") {
        Ok(token) => token,
        Err(_) => {
            log::error!("Failed to retrieve auth token from keychain");
            let response = TokenResponse {
                status: "error".to_string(),
                message: "Server configuration error".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    if auth_token != expected_token {
        log::warn!("Invalid auth token from IP: {}", client_ip);
        let response = TokenResponse {
            status: "error".to_string(),
            message: "Invalid authentication token".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, Json(response));
    }

    // Validate code format (6 digits)
    if !payload.code.chars().all(|c| c.is_ascii_digit()) || payload.code.len() != 6 {
        log::warn!(
            "Invalid code format from IP: {}: '{}'",
            client_ip,
            payload.code
        );
        let response = TokenResponse {
            status: "error".to_string(),
            message: "Invalid code format (must be 6 digits)".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    log::info!("Processing valid code from IP: {}", client_ip);

    // Get VPN password
    let password = match crate::security::KeychainManager::get_password("vpn_password") {
        Ok(pwd) => pwd,
        Err(_) => {
            log::error!("Failed to retrieve VPN password from keychain");
            let response = TokenResponse {
                status: "error".to_string(),
                message: "Server configuration error".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    // Start VPN connection in background
    let vpn_code = payload.code.clone();
    let vpn_automator = Arc::clone(&state.vpn_automator);
    tokio::spawn(async move {
        log::info!("Starting VPN connection for code: {}", vpn_code);
        match vpn_automator.connect(&vpn_code, &password) {
            Ok(_) => {
                log::info!("VPN connection successful for code: {}", vpn_code);
            }
            Err(e) => {
                log::error!("VPN connection failed for code {}: {:?}", vpn_code, e);
            }
        }
    });

    // Return response immediately
    log::info!("VPN connection initiated for code from IP: {}", client_ip);
    let response = TokenResponse {
        status: "accepted".to_string(),
        message: "VPN connection initiated".to_string(),
    };
    (StatusCode::ACCEPTED, Json(response))
}
