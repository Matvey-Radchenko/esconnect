# ESConnect — автоматизация Endpoint Security VPN

Автоматизация подключения к корпоративному Endpoint Security VPN (Check Point Endpoint Security).

---

## Требования

- macOS
- Android-телефон с приложением [MacroDroid](https://play.google.com/store/apps/details?id=com.arlosoft.macrodroid)
- Приложение **Indeed Key** на телефоне

---

## Установка

### 0. Загружаем проект

```bash
git clone <repository-url>
cd esconnect
```

### 1. Телефон

1. Установить [MacroDroid](https://play.google.com/store/apps/details?id=com.arlosoft.macrodroid)
2. Импортировать файл `Indeed_Key_HTTP.macro` из папки проекта
3. Получить **url webhook'а** для будущей установки скрипта на маке

### 2. Mac

```bash
./install.sh
```

Установщик:
- Скопирует бинарник в `/usr/local/bin/esconnect`
- Предложит запустить интерактивную настройку (`esconnect setup`)
- В процессе `setup` будет сгенерирован **Auth Token** — он понадобится на следующем шаге.
- Вводим **url webhook'а**
- Запустится демон

### 3. Телефон

В макросе настроить два поля:
   - **Шаг "PIN Unlock"** — вставить PIN своего телефона (или удалить шаг, если телефон без блокировки)
   - **Шаг "HTTP Request" → заголовок `X-Auth-Token`** — вставить токен из `esconnect setup`

Если используете VPN с раздельным туннелированием - добавьте macrodroid в исключения. Отправка кодов работает только в локальной сети => с VPN работать не будет

---

Скрипт сам запросит нужные разрешения на работу у MacOS при первой попытке подключения. Но можно их выдать заранее в **System Settings → Privacy & Security**:

| Разрешение       | Зачем                                        |
|------------------|----------------------------------------------|
| Accessibility    | Управление интерфейсом VPN через System Events |
| Input Monitoring | Ввод OTP и пароля в поля VPN                  |



---

## Использование

```bash
esconnect connect      # Запросить OTP и подключиться
esconnect disconnect   # Отключиться
esconnect toggle       # Переключить состояние (для настроек горячих клавиш, например через raycast)
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

---

## Диагностика

```bash
esconnect status
tail -f /tmp/esconnect.log
```
