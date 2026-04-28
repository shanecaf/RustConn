# Задачі: Hoop.dev ZeroTrust Provider

## Задача 1: Зміни моделі даних (rustconn-core)

- [x] 1.1 Додати варіант `HoopDev` до enum `ZeroTrustProvider` у `rustconn-core/src/models/protocol.rs`
  - `#[serde(rename = "hoop_dev")]` атрибут
  - `display_name()` → `"Hoop.dev"`
  - `icon_name()` → `"network-transmit-symbolic"` (унікальна іконка)
  - `cli_command()` → `"hoop"`
  - Додати до `all()` перед `Generic`
  - Додати до `Display` impl
  - _Вимоги: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 1.2 Створити структуру `HoopDevConfig` у `rustconn-core/src/models/protocol.rs`
  - Поля: `connection_name: String`, `gateway_url: Option<String>`, `grpc_url: Option<String>`
  - Derive: `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`
  - `#[serde(skip_serializing_if = "Option::is_none")]` для опціональних полів
  - Додати варіант `HoopDev(HoopDevConfig)` до enum `ZeroTrustProviderConfig`
  - _Вимоги: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [x] 1.3 Додати валідацію `HoopDev` у `ZeroTrustConfig::validate()`
  - Новий match-arm: перевірка що `connection_name.trim()` не порожній
  - Повертати `ProtocolError::InvalidConfig` при порожньому імені
  - _Вимоги: 3.1, 3.2, 3.3, 3.4_

- [x] 1.4 Додати генерацію команди `HoopDev` у `ZeroTrustConfig::build_command()`
  - Команда: `hoop connect <connection_name>`
  - Опціонально: `--api-url <gateway_url>`, `--grpc-url <grpc_url>`
  - Custom args додаються загальним кодом після match-блоку
  - _Вимоги: 4.1, 4.2, 4.3, 4.4_

## Задача 2: Детекція CLI (rustconn-core)

- [x] 2.1 Додати функцію `detect_hoop()` у `rustconn-core/src/protocol/detection.rs`
  - За патерном `detect_boundary()`: виклик `detect_client("Hoop.dev", "hoop", &["version"], "Install: https://hoop.dev/docs/installing")`
  - Додати поле `pub hoop: ClientInfo` до `ZeroTrustDetectionResult`
  - Додати виклик `detect_hoop()` у `detect_all()`
  - Додати `&self.hoop` у `as_vec()`
  - _Вимоги: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

## Задача 3: Завантажуваний компонент Flatpak (rustconn-core)

- [x] 3.1 Додати `DownloadableComponent` для `hoop` у масив `DOWNLOADABLE_COMPONENTS` у `rustconn-core/src/cli_download.rs`
  - Позиція: після `boundary`, перед Password Manager CLIs
  - `id: "hoop"`, `name: "Hoop.dev"`, `category: ComponentCategory::ZeroTrust`
  - `binary_name: "hoop"`, `install_subdir: "hoop"`, `works_in_sandbox: true`
  - URL завантаження для x86_64 та aarch64
  - `checksum: ChecksumPolicy::SkipLatest`
  - _Вимоги: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

## Задача 4: Контрольна точка — Перевірка ядра

- [x] 4. Переконатися що всі тести проходять, запитати користувача якщо є питання.

## Задача 5: GUI-поля у діалозі з'єднання (rustconn)

- [x] 5.1 Створити функцію `create_hoop_dev_fields()` у `rustconn/src/dialogs/connection/zerotrust.rs`
  - `connection_name`: обов'язкове `adw::EntryRow` з label `i18n("Connection Name")`, placeholder `i18n("e.g., my-database")`
  - `gateway_url`: опціональне `adw::EntryRow` з label `i18n("Gateway URL")`, placeholder `i18n("e.g., https://app.hoop.dev")`
  - `grpc_url`: опціональне `adw::EntryRow` з label `i18n("gRPC URL")`, placeholder `i18n("e.g., grpc.hoop.dev:8443")`
  - _Вимоги: 7.2, 7.3, 7.4, 7.7_

- [x] 5.2 Оновити `create_zerotrust_options()` та dropdown провайдерів
  - Додати `"Hoop.dev"` у dropdown перед `"Generic Command"`
  - Додати поля до `ZeroTrustOptionsWidgets` tuple
  - _Вимоги: 7.1_

- [x] 5.3 Оновити `dialog.rs` для маппінгу HoopDev
  - Маппінг індексу dropdown ↔ `ZeroTrustProvider::HoopDev` (індекс 9, Generic стає 10)
  - Збирання `HoopDevConfig` з полів при збереженні
  - Заповнення полів при відкритті існуючого з'єднання
  - _Вимоги: 7.5, 7.6_

## Задача 6: CLI підтримка (rustconn-cli)

- [x] 6.1 Оновити `show.rs` для відображення полів HoopDev
  - Додати match-arm для `ZeroTrustProviderConfig::HoopDev` у виводі `show`
  - Відображати `connection_name`, `gateway_url`, `grpc_url`
  - _Вимоги: 8.5_

- [x] 6.2 Оновити CLI-аргументи для HoopDev у `rustconn-cli/src/cli.rs`
  - Додати `hoop_dev` як валідне значення `--provider`
  - Додати аргументи: `--hoop-connection-name`, `--hoop-gateway-url`, `--hoop-grpc-url`
  - Оновити обробники `Add` та `Update` команд
  - _Вимоги: 8.1, 8.2, 8.3, 8.4_

## Задача 7: Дозволи Flatpak-маніфестів

- [x] 7.1 Додати `--filesystem=home/.hoop:ro` до Flatpak-маніфестів
  - `packaging/flatpak/io.github.totoshko88.RustConn.yml`
  - `packaging/flatpak/io.github.totoshko88.RustConn.local.yml`
  - `packaging/flathub/io.github.totoshko88.RustConn.yml`
  - Додати коментар: `# Hoop.dev CLI config and access tokens`
  - Позиція: після рядка `--filesystem=home/.kube:ro`
  - _Вимоги: 9.1, 9.2, 9.3, 9.4_

## Задача 8: Контрольна точка — Перевірка інтеграції

- [x] 8. Переконатися що всі тести проходять, запитати користувача якщо є питання.

## Задача 9: Property-тести та unit-тести

- [x] 9.1 Додати стратегію `arb_hoop_dev_config()` для генерації довільних `HoopDevConfig`
  - `connection_name`: непорожній рядок `[a-zA-Z0-9_-]{1,50}`
  - `gateway_url`: `Option<String>` — `None` або URL-подібний рядок
  - `grpc_url`: `Option<String>` — `None` або host:port рядок
  - _Вимоги: 12.1_

- [x] 9.2 Property-тест: Round-trip серіалізація HoopDevConfig
  - **Property 1: HoopDevConfig serialization round-trip**
  - **Перевіряє: Вимоги 2.4, 10.1, 10.2, 10.3, 12.3**

- [x] 9.3 Property-тест: None-поля відсутні у серіалізованому JSON
  - **Property 2: None fields omitted from serialized JSON**
  - **Перевіряє: Вимоги 2.5**

- [x] 9.4 Property-тест: Валідація приймає валідні та відхиляє порожні connection_name
  - **Property 3: Validation accepts valid configs and rejects empty connection_name**
  - **Перевіряє: Вимоги 3.1, 3.2**

- [x] 9.5 Property-тест: Коректність генерації команди
  - **Property 4: Command generation correctness**
  - **Перевіряє: Вимоги 4.1, 4.2, 4.3, 4.4**

- [x] 9.6 Property-тест: Унікальність іконок провайдерів
  - **Property 5: Provider icon names are unique**
  - Оновити існуючий `prop_protocol_icons_are_distinct` для включення HoopDev
  - **Перевіряє: Вимоги 1.3, 12.4**

- [x] 9.7 Unit-тести для HoopDev у `protocol.rs`
  - `test_hoop_dev_serde_rename` — перевірка serde rename = "hoop_dev"
  - `test_hoop_dev_display_name` — перевірка display_name() = "Hoop.dev"
  - `test_hoop_dev_cli_command` — перевірка cli_command() = "hoop"
  - `test_hoop_dev_in_all` — перевірка що HoopDev є в all() перед Generic
  - `test_hoop_dev_validate_empty_name` — порожній connection_name → Err
  - `test_hoop_dev_validate_valid` — валідний config → Ok
  - `test_hoop_dev_build_command_basic` — базова команда
  - `test_hoop_dev_build_command_with_urls` — команда з gateway_url та grpc_url
  - `test_hoop_dev_build_command_with_custom_args` — команда з custom_args
  - _Вимоги: 1.1, 1.2, 1.4, 1.5, 3.1, 3.2, 4.1, 4.2, 4.3, 4.4, 12.4, 12.5_

- [x] 9.8 Unit-тест для детекції та Flatpak-компонента
  - `test_hoop_detection_returns_valid_info` — detect_hoop() повертає валідний ClientInfo
  - `test_hoop_downloadable_component` — перевірка полів DownloadableComponent для hoop
  - Додати `arb_hoop_command()` до `arb_command_with_provider()` для тестування детекції провайдера
  - _Вимоги: 5.1, 6.1, 12.1, 12.2_

## Задача 10: Інтернаціоналізація (i18n)

- [x] 10.1 Оновити `po/rustconn.pot` через `po/update-pot.sh`
  - Переконатися що всі нові рядки з `i18n()` потрапили до шаблону перекладів
  - _Вимоги: 11.1, 11.2, 11.3, 11.4_

## Задача 11: Фінальна контрольна точка

- [x] 11. Переконатися що всі тести проходять, запитати користувача якщо є питання.

## Примітки

- Задачі позначені `*` є опціональними і можуть бути пропущені для швидшого MVP
- Кожна задача посилається на конкретні вимоги для трасування
- Контрольні точки забезпечують інкрементальну валідацію
- Property-тести перевіряють універсальні властивості коректності
- Unit-тести перевіряють конкретні приклади та граничні випадки
