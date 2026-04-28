# Документ вимог

## Вступ

Hoop.dev — це zero-trust шлюз доступу до баз даних та серверів. Він надає CLI-інструмент `hoop`, який працює як проксі між користувачами та інфраструктурою, забезпечуючи SSO/OIDC автентифікацію через браузер, інтерактивні сесії (`hoop connect`), одноразові команди (`hoop exec`), маскування даних та аудит-логування.

Ця функція додає Hoop.dev як 11-го ZeroTrust-провайдера в RustConn, дотримуючись існуючих патернів для AWS SSM, GCP IAP, Azure Bastion, Cloudflare Access, Teleport, Tailscale SSH, HashiCorp Boundary та інших провайдерів. Реалізація охоплює модель даних, генерацію команд, детекцію CLI, завантаження CLI для Flatpak, GUI-поля, CLI-підтримку, серіалізацію, Flatpak-дозволи, інтернаціоналізацію та property-тести.

## Глосарій

- **ZeroTrustProvider**: Enum у `rustconn-core/src/models/protocol.rs`, що перелічує всіх підтримуваних ZeroTrust-провайдерів (AWS SSM, GCP IAP, Azure Bastion тощо).
- **ZeroTrustProviderConfig**: Enum у `rustconn-core/src/models/protocol.rs`, що містить provider-specific конфігурації для кожного ZeroTrust-провайдера.
- **HoopDevConfig**: Структура конфігурації для Hoop.dev провайдера, що містить назву з'єднання, URL шлюзу та gRPC URL.
- **ZeroTrustConfig**: Структура у `rustconn-core/src/models/protocol.rs`, що об'єднує провайдера, provider-specific конфігурацію та додаткові аргументи.
- **Connection_Dialog**: Діалог створення/редагування з'єднання (`rustconn/src/dialogs/connection/`).
- **ZeroTrust_Tab**: Панель ZeroTrust-опцій у Connection_Dialog (`rustconn/src/dialogs/connection/zerotrust.rs`).
- **Detection_Module**: Модуль детекції CLI-інструментів (`rustconn-core/src/protocol/detection.rs`).
- **ZeroTrustDetectionResult**: Структура результатів детекції всіх ZeroTrust CLI-інструментів.
- **CLI_Download_Module**: Модуль завантаження CLI для Flatpak-середовища (`rustconn-core/src/cli_download.rs`).
- **DownloadableComponent**: Структура, що описує завантажуваний CLI-компонент для Flatpak.
- **CLI**: Крейт `rustconn-cli`, що надає командний рядок для керування з'єднаннями.
- **Flatpak_Manifest**: Файли маніфесту Flatpak (`packaging/flatpak/*.yml`, `packaging/flathub/*.yml`), що декларують дозволи пісочниці.
- **Native_Export**: Модуль нативного експорту/імпорту з'єднань (`rustconn-core/src/export/native.rs`).
- **Hoop_CLI**: Командний рядок `hoop` від Hoop.dev для підключення до інфраструктури.
- **Connection_Name**: Ідентифікатор з'єднання в Hoop.dev (передається як аргумент `hoop connect <connection-name>`).
- **Gateway_URL**: URL API-шлюзу Hoop.dev (зберігається в `~/.hoop/config.toml` як `api_url`).
- **GRPC_URL**: URL gRPC-сервера Hoop.dev (зберігається в `~/.hoop/config.toml` як `grpc_url`).

## Вимоги

### Вимога 1: Варіант HoopDev у моделі ZeroTrustProvider

**User Story:** Як розробник RustConn, я хочу додати варіант `HoopDev` до enum `ZeroTrustProvider`, щоб Hoop.dev був доступний як ZeroTrust-провайдер у всій кодовій базі.

#### Критерії прийняття

1. THE ZeroTrustProvider SHALL include a `HoopDev` variant with serde rename `hoop_dev`
2. THE ZeroTrustProvider `display_name()` method SHALL return `"Hoop.dev"` for the `HoopDev` variant
3. THE ZeroTrustProvider `icon_name()` method SHALL return a unique Adwaita symbolic icon name for the `HoopDev` variant that does not duplicate any existing provider icon
4. THE ZeroTrustProvider `cli_command()` method SHALL return `"hoop"` for the `HoopDev` variant
5. THE ZeroTrustProvider `all()` method SHALL include `HoopDev` in the returned slice, positioned before `Generic`

### Вимога 2: Структура HoopDevConfig

**User Story:** Як розробник RustConn, я хочу мати структуру `HoopDevConfig` з полями конфігурації Hoop.dev, щоб зберігати параметри з'єднання для цього провайдера.

#### Критерії прийняття

1. THE HoopDevConfig SHALL contain a required `connection_name` field of type `String` representing the Hoop.dev Connection_Name
2. THE HoopDevConfig SHALL contain an optional `gateway_url` field of type `Option<String>` representing the Gateway_URL
3. THE HoopDevConfig SHALL contain an optional `grpc_url` field of type `Option<String>` representing the GRPC_URL
4. THE HoopDevConfig SHALL derive `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize`
5. THE HoopDevConfig serializer SHALL omit `gateway_url` and `grpc_url` fields when they are `None`
6. THE ZeroTrustProviderConfig SHALL include a `HoopDev(HoopDevConfig)` variant

### Вимога 3: Валідація конфігурації HoopDev

**User Story:** Як користувач, я хочу отримувати зрозумілі повідомлення про помилки при некоректній конфігурації Hoop.dev, щоб я міг виправити налаштування перед підключенням.

#### Критерії прийняття

1. WHEN the `connection_name` field in HoopDevConfig is empty, THE ZeroTrustConfig `validate()` method SHALL return a `ProtocolError::InvalidConfig` error indicating that the connection name is required
2. WHEN the `connection_name` field in HoopDevConfig is non-empty, THE ZeroTrustConfig `validate()` method SHALL return `Ok(())`
3. WHEN the `gateway_url` field contains a non-empty value, THE ZeroTrustConfig `validate()` method SHALL accept the configuration without error
4. WHEN the `grpc_url` field contains a non-empty value, THE ZeroTrustConfig `validate()` method SHALL accept the configuration without error

### Вимога 4: Генерація команди підключення

**User Story:** Як користувач, я хочу підключатися до Hoop.dev-ресурсів через RustConn, щоб мати інтерактивну сесію через `hoop connect`.

#### Критерії прийняття

1. WHEN the ZeroTrust provider is `HoopDev`, THE command builder SHALL generate the command `hoop connect <connection_name>`
2. WHEN the HoopDevConfig contains a non-empty `gateway_url`, THE command builder SHALL include `--api-url <gateway_url>` in the generated command arguments
3. WHEN the HoopDevConfig contains a non-empty `grpc_url`, THE command builder SHALL include `--grpc-url <grpc_url>` in the generated command arguments
4. WHEN the ZeroTrustConfig contains non-empty `custom_args`, THE command builder SHALL append the custom arguments after the provider-specific arguments
5. THE command builder SHALL use the `format_connection_message()` function with protocol name `"Hoop.dev"` and the Connection_Name as host identifier

### Вимога 5: Детекція CLI hoop

**User Story:** Як користувач, я хочу бачити статус встановлення CLI `hoop` у RustConn, щоб знати, чи готовий мій інструмент до використання.

#### Критерії прийняття

1. THE Detection_Module SHALL include a `detect_hoop()` function that returns a `ClientInfo` for the `hoop` binary
2. THE `detect_hoop()` function SHALL search for the `hoop` binary using the same mechanism as other ZeroTrust CLI detection functions (`detect_client()`)
3. THE `detect_hoop()` function SHALL extract the version string from the `hoop` binary output
4. THE ZeroTrustDetectionResult SHALL include a `hoop` field of type `ClientInfo`
5. THE ZeroTrustDetectionResult `detect_all()` method SHALL call `detect_hoop()` and store the result in the `hoop` field
6. THE ZeroTrustDetectionResult `as_vec()` method SHALL include the `hoop` ClientInfo in the returned vector

### Вимога 6: Завантаження CLI для Flatpak

**User Story:** Як користувач Flatpak, я хочу мати можливість завантажити CLI `hoop` через менеджер компонентів RustConn, щоб використовувати Hoop.dev без ручного встановлення.

#### Критерії прийняття

1. THE CLI_Download_Module SHALL include a DownloadableComponent entry for the `hoop` CLI with `id` set to `"hoop"`
2. THE DownloadableComponent for `hoop` SHALL have `category` set to `ComponentCategory::ZeroTrust`
3. THE DownloadableComponent for `hoop` SHALL have `binary_name` set to `"hoop"`
4. THE DownloadableComponent for `hoop` SHALL have `download_url` pointing to the official Hoop.dev Linux x86_64 binary release URL
5. THE DownloadableComponent for `hoop` SHALL have `works_in_sandbox` set to `true`
6. THE DownloadableComponent for `hoop` SHALL have a `ChecksumPolicy` with a SHA256 checksum for download verification

### Вимога 7: GUI-поля у діалозі з'єднання

**User Story:** Як користувач GUI, я хочу бачити поля конфігурації Hoop.dev у діалозі ZeroTrust-з'єднання, щоб налаштувати підключення через графічний інтерфейс.

#### Критерії прийняття

1. THE ZeroTrust_Tab SHALL include a `"Hoop.dev"` option in the provider dropdown, positioned before `"Generic Command"`
2. WHEN the user selects `"Hoop.dev"` in the provider dropdown, THE ZeroTrust_Tab SHALL display a provider-specific fields panel with an `adw::EntryRow` for Connection_Name
3. WHEN the user selects `"Hoop.dev"` in the provider dropdown, THE ZeroTrust_Tab SHALL display an optional `adw::EntryRow` for Gateway_URL
4. WHEN the user selects `"Hoop.dev"` in the provider dropdown, THE ZeroTrust_Tab SHALL display an optional `adw::EntryRow` for GRPC_URL
5. WHEN the user opens the Connection_Dialog for an existing Hoop.dev connection, THE ZeroTrust_Tab SHALL populate the Connection_Name, Gateway_URL, and GRPC_URL fields with the saved HoopDevConfig values
6. WHEN the user saves a Hoop.dev connection, THE Connection_Dialog SHALL construct a `HoopDevConfig` from the entered field values and store it in `ZeroTrustProviderConfig::HoopDev`
7. THE ZeroTrustOptionsWidgets tuple SHALL include `adw::EntryRow` fields for `hoop_connection_name`, `hoop_gateway_url`, and `hoop_grpc_url`

### Вимога 8: Підтримка CLI rustconn-cli

**User Story:** Як користувач CLI, я хочу створювати та оновлювати Hoop.dev з'єднання через `rustconn-cli`, щоб мати паритет функціональності з GUI.

#### Критерії прийняття

1. THE CLI SHALL recognize `hoop_dev` as a valid value for the `--provider` argument in ZeroTrust connection commands
2. THE CLI SHALL accept `--hoop-connection-name <NAME>` argument for specifying the Hoop.dev Connection_Name
3. THE CLI SHALL accept optional `--hoop-gateway-url <URL>` argument for specifying the Gateway_URL
4. THE CLI SHALL accept optional `--hoop-grpc-url <URL>` argument for specifying the GRPC_URL
5. THE CLI `show` command SHALL display HoopDev-specific fields (connection_name, gateway_url, grpc_url) when showing a Hoop.dev connection

### Вимога 9: Дозволи Flatpak для конфігурації Hoop.dev

**User Story:** Як користувач Flatpak, я хочу щоб RustConn мав доступ до конфігурації `~/.hoop/`, щоб CLI `hoop` міг читати токени автентифікації та налаштування шлюзу.

#### Критерії прийняття

1. THE Flatpak_Manifest SHALL include `--filesystem=home/.hoop:ro` to allow read-only access to the Hoop.dev configuration directory
2. THE Flatpak_Manifest changes SHALL be applied to both `packaging/flatpak/io.github.totoshko88.RustConn.yml` and `packaging/flatpak/io.github.totoshko88.RustConn.local.yml`
3. THE Flatpak_Manifest changes SHALL be applied to `packaging/flathub/io.github.totoshko88.RustConn.yml`
4. THE Flatpak_Manifest SHALL include a comment explaining the purpose of the `home/.hoop:ro` permission (Hoop.dev CLI config and access tokens)

### Вимога 10: Серіалізація та імпорт/експорт

**User Story:** Як користувач, я хочу експортувати та імпортувати Hoop.dev з'єднання у нативному форматі RustConn, щоб переносити конфігурацію між машинами.

#### Критерії прийняття

1. WHEN a connection with `ZeroTrustProviderConfig::HoopDev` is serialized to JSON, THE Native_Export SHALL include all HoopDevConfig fields in the output
2. WHEN a JSON file containing a `hoop_dev` provider configuration is imported, THE Native_Export SHALL deserialize it into `ZeroTrustProviderConfig::HoopDev(HoopDevConfig)`
3. FOR ALL valid HoopDevConfig values, serializing to JSON then deserializing SHALL produce an equivalent HoopDevConfig (round-trip property)

### Вимога 11: Інтернаціоналізація

**User Story:** Як не-англомовний користувач, я хочу бачити всі нові рядки інтерфейсу Hoop.dev перекладеними моєю мовою, щоб функція була доступною для всіх.

#### Критерії прийняття

1. THE ZeroTrust_Tab SHALL wrap all user-visible strings for Hoop.dev fields (labels, placeholders, descriptions) in the `i18n()` macro
2. THE ZeroTrust_Tab SHALL wrap parameterized strings in the `i18n_f()` macro where applicable
3. WHEN new translatable strings are added, THE `po/rustconn.pot` template SHALL be updated to include the new strings
4. THE Connection_Dialog SHALL wrap all Hoop.dev-related validation error messages in the `i18n()` macro

### Вимога 12: Property-тести

**User Story:** Як розробник, я хочу мати property-тести для HoopDev, щоб забезпечити коректність серіалізації, валідації та генерації команд.

#### Критерії прийняття

1. THE property test strategies SHALL include an `arb_hoop_command()` strategy that generates valid `hoop connect` command strings
2. THE property test strategies SHALL include HoopDev in the `arb_command_with_provider()` strategy for provider detection tests
3. THE serialization property tests SHALL verify round-trip serialization for `ZeroTrustProviderConfig::HoopDev` with arbitrary HoopDevConfig values
4. THE protocol property tests SHALL verify that `HoopDev` has a unique icon name distinct from all other providers
5. THE protocol property tests SHALL verify that `HoopDev` is included in `ZeroTrustProvider::all()`
