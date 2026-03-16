# ESConnect — автоматизация Endpoint Security VPN

Автоматически подключается к корпоративному Endpoint Security VPN (Check Point).

При запросе кода — звонит на телефон через MacroDroid, тот открывает Indeed Key, считывает OTP и отправляет его обратно на Mac.

---

## Требования

- macOS (Sequoia и новее)
- Android-телефон с приложением [MacroDroid](https://play.google.com/store/apps/details?id=com.arlosoft.macrodroid)
- Приложение **Indeed Key** на телефоне

---

## Установка

### 1. Mac

```bash
git clone <repository-url>
cd esconnect
./install.sh
```

Установщик:
- Скопирует бинарник в `/usr/local/bin/esconnect`
- Запустит интерактивную настройку (`esconnect setup`)
- Запустит демон

В процессе `setup` будет сгенерирован **Auth Token** — он понадобится на следующем шаге.

После установки выдать разрешения в **System Settings → Privacy & Security**:

| Разрешение       | Зачем                                        |
|------------------|----------------------------------------------|
| Accessibility    | Управление интерфейсом VPN через System Events |
| Input Monitoring | Ввод OTP и пароля в поля VPN                  |

### 2. Телефон

1. Установить [MacroDroid](https://play.google.com/store/apps/details?id=com.arlosoft.macrodroid)
2. Импортировать файл `Indeed_Key_HTTP.macro` из этого репозитория
3. В макросе настроить два поля:
   - **Шаг "PIN Unlock"** — вставить PIN своего телефона (или удалить шаг, если телефон без блокировки)
   - **Шаг "HTTP Request" → заголовок `X-Auth-Token`** — вставить токен из `esconnect setup`
4. Включить макрос

---

## Использование

```bash
esconnect connect      # Запросить OTP и подключиться
esconnect disconnect   # Отключиться
esconnect toggle       # Переключить состояние
esconnect status       # Статус VPN и демона
esconnect setup        # Изменить настройки
esconnect start        # Запустить демон
esconnect stop         # Остановить демон
esconnect token        # Показать auth token
```

---

## Как это работает

```
esconnect connect
  └─→ вызывает webhook MacroDroid с ?ip=<локальный IP>

MacroDroid (телефон)
  └─→ будит экран
  └─→ открывает Indeed Key
  └─→ считывает OTP с экрана
  └─→ POST http://<IP>:8337/token {"code": "123456"}

Демон (Mac)
  └─→ проверяет токен и IP
  └─→ открывает меню VPN
  └─→ вводит OTP и пароль
  └─→ подключается
```

---

## Настройка

```bash
esconnect setup
```

Доступные параметры:
- **Auth Token** — токен для аутентификации запросов с телефона
- **Server Subnet** — разрешённая подсеть (определяется автоматически)
- **MacroDroid Webhook** — URL для вызова телефона
- **VPN Password** — сохраняется в системном Keychain
- **Delays** — тайминги автоматизации (если VPN тормозит)

---

## Диагностика

```bash
esconnect status
tail -f /tmp/esconnect.log
```
