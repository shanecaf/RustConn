# План рефакторингу RustConn

**Дата:** 2026-05-11  
**Базується на:** Аудит + Peer Review (скориговані пріоритети)

---

## Фаза 1 — Безпечні виправлення (без ризику регресії)

### 1.1 Виправити `expect()` у `csv_import.rs:373`

**Проблема:** `parent_id.expect("group path should have at least one segment")` — паніка при порожньому шляху групи з некоректних CSV-даних.

**Поточний код:**
```rust
// rustconn-core/src/import/csv_import.rs:373
parent_id.expect("group path should have at least one segment")
```

**Виправлення:**
```rust
// rustconn-core/src/import/csv_import.rs
parent_id.ok_or_else(|| {
    ImportError::InvalidData("group path must contain at least one segment".to_string())
})?
```

**Зміна сигнатури функції:**
```rust
// Було:
fn ensure_group_hierarchy(
    path: &str,
    groups: &mut HashMap<String, Uuid>,
    result: &mut ImportResult,
) -> Uuid

// Стало:
fn ensure_group_hierarchy(
    path: &str,
    groups: &mut HashMap<String, Uuid>,
    result: &mut ImportResult,
) -> Result<Uuid, ImportError>
```

**Вплив:** Мінімальний — функція викликається лише всередині `csv_import.rs`. Потрібно додати `?` у місці виклику.

---

### 1.2 Розбити `performance/mod.rs` (2 210 рядків → 5 модулів)

**Поточна структура:** Один файл містить 15+ незалежних структур.

**Цільова структура:**
```
rustconn-core/src/performance/
├── mod.rs              (~100 рядків — re-exports, lock helpers, global metrics())
├── metrics.rs          (~250 рядків — PerformanceMetrics, OperationStats, TimingGuard)
├── debouncer.rs        (~100 рядків — Debouncer)
├── memory.rs           (~500 рядків — MemoryTracker, MemoryOptimizer, MemoryEstimate, MemorySnapshot, MemoryPressure, MemoryBreakdown)
├── batch.rs            (~150 рядків — BatchProcessor)
├── pool.rs             (~150 рядків — ObjectPool, PoolStats)
├── compact_string.rs   (~150 рядків — CompactString, CompactStringStorage)
├── interner.rs         (~120 рядків — StringInterner, InternerStats)
├── scroller.rs         (~100 рядків — VirtualScroller)
├── shrinkable_vec.rs   (~120 рядків — ShrinkableVec)
└── lazy.rs             (~50 рядків — LazyInit)
```

**Приклад `mod.rs` після рефакторингу:**
```rust
//! Performance utilities: metrics, memory tracking, batching, and data structures.

mod batch;
mod compact_string;
mod debouncer;
mod interner;
mod lazy;
mod memory;
mod metrics;
mod pool;
mod scroller;
mod shrinkable_vec;

use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub use batch::BatchProcessor;
pub use compact_string::CompactString;
pub use debouncer::Debouncer;
pub use interner::{InternerStats, StringInterner};
pub use lazy::LazyInit;
pub use memory::{
    AllocationStats, MemoryBreakdown, MemoryEstimate, MemoryOptimizer, MemoryPressure,
    MemorySnapshot, MemoryTracker, OptimizationCategory, OptimizationRecommendation,
};
pub use metrics::{OperationStats, PerformanceMetrics, TimingGuard};
pub use pool::{ObjectPool, PoolStats};
pub use scroller::VirtualScroller;
pub use shrinkable_vec::ShrinkableVec;

pub(crate) fn lock_mutex<'a, T>(mutex: &'a Mutex<T>, name: &str) -> Option<MutexGuard<'a, T>> {
    // ... існуючий код
}

pub(crate) fn read_rwlock<'a, T>(lock: &'a RwLock<T>, name: &str) -> Option<RwLockReadGuard<'a, T>> {
    // ...
}

pub(crate) fn write_rwlock<'a, T>(lock: &'a RwLock<T>, name: &str) -> Option<RwLockWriteGuard<'a, T>> {
    // ...
}

/// Global performance metrics singleton.
pub fn metrics() -> &'static PerformanceMetrics {
    static METRICS: std::sync::OnceLock<PerformanceMetrics> = std::sync::OnceLock::new();
    METRICS.get_or_init(PerformanceMetrics::new)
}

/// Global memory optimizer singleton.
pub fn memory_optimizer() -> &'static MemoryOptimizer {
    static OPTIMIZER: std::sync::OnceLock<MemoryOptimizer> = std::sync::OnceLock::new();
    OPTIMIZER.get_or_init(MemoryOptimizer::new)
}

/// Formats byte count as human-readable string.
pub fn format_bytes(bytes: usize) -> String {
    // ... існуючий код
}
```

**Порядок дій:**
1. Створити підмодулі, перенести структури (без зміни логіки)
2. Оновити `mod.rs` — тільки re-exports
3. `cargo test -p rustconn-core` — переконатися, що тести проходять
4. Зовнішній API не змінюється (всі `pub use` залишаються)

---

### 1.3 Розбити `cli_download.rs` (3 391 рядків → модуль)

**Цільова структура:**
```
rustconn-core/src/cli_download/
├── mod.rs              (~400 рядків — types, public API, component registry)
├── components.rs       (~200 рядків — COMPONENTS static array, get_component(), get_available_components())
├── detection.rs        (~100 рядків — detect_package_manager(), get_system_install_command())
├── download.rs         (~100 рядків — download_with_progress(), verify_checksum())
├── extract.rs          (~300 рядків — extract_zip, extract_tar, extract_deb, etc.)
├── install_pip.rs      (~250 рядків — install_pip_component, create_pip_wrapper_script, ensure_pip_available)
├── install_custom.rs   (~600 рядків — install_kubectl, install_teleport, install_tailscale, install_boundary, install_hoop)
├── install_cloud.rs    (~300 рядків — install_gcloud, install_aws_cli)
├── install_secrets.rs  (~200 рядків — install_bitwarden, install_1password)
├── uninstall.rs        (~100 рядків — uninstall_component)
└── update.rs           (~100 рядків — update_component, update_pip_component)
```

**Принцип:** Кожен файл — одна відповідальність. Публічний API залишається незмінним через re-exports у `mod.rs`.

**Приклад `mod.rs`:**
```rust
//! CLI tool download, installation, and management.
//!
//! Supports downloading cloud CLIs (aws, gcloud, az, oci),
//! secret managers (bitwarden, 1password), and infrastructure tools
//! (kubectl, teleport, tailscale, boundary, hoop).

mod components;
mod detection;
mod download;
mod extract;
mod install_cloud;
mod install_custom;
mod install_pip;
mod install_secrets;
mod uninstall;
mod update;

// Re-export public API
pub use components::{get_available_components, get_component, get_components_by_category, get_installation_status, get_pinned_versions};
pub use detection::{detect_package_manager, get_system_install_command, PackageManager};
pub use download::DownloadProgress;
pub use uninstall::uninstall_component;
pub use update::update_component;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Cancellation token for download operations.
#[derive(Debug, Clone)]
pub struct DownloadCancellation { /* ... */ }

/// Errors during CLI download/install operations.
#[derive(Debug, thiserror::Error)]
pub enum CliDownloadError { /* ... */ }

// ... решта типів та публічних функцій
```

---

## Фаза 2 — Інтеграція підготовленого API

### 2.1 Інтегрувати `RetryConfig`/`RetryState` у GUI

**Контекст:** Механізм retry повністю реалізований і протестований, але не підключений до UI. Користувач не бачить автоматичного reconnect з backoff.

**План інтеграції:**

**Крок 1 — Додати поле до `Connection`:**
```rust
// rustconn-core/src/models/connection.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    // ... існуючі поля
    /// Retry configuration for automatic reconnection
    #[serde(default)]
    pub retry_config: RetryConfig,
}
```

**Крок 2 — UI в ConnectionDialog (вкладка "Advanced"):**
```rust
// rustconn/src/dialogs/connection/advanced.rs
fn build_retry_section(config: &RetryConfig) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(&i18n("Automatic Reconnection"))
        .build();

    let enabled_row = adw::SwitchRow::builder()
        .title(&i18n("Enable auto-reconnect"))
        .active(config.enabled)
        .build();

    let attempts_row = adw::SpinRow::builder()
        .title(&i18n("Maximum attempts"))
        .adjustment(&gtk4::Adjustment::new(
            f64::from(config.max_attempts), 1.0, 10.0, 1.0, 1.0, 0.0,
        ))
        .build();

    group.add(&enabled_row);
    group.add(&attempts_row);
    group
}
```

**Крок 3 — Обробка розриву з'єднання:**
```rust
// rustconn/src/window/session_lifecycle.rs
fn handle_connection_lost(
    state: &SharedAppState,
    notebook: &SharedNotebook,
    session_id: Uuid,
    connection_id: Uuid,
) {
    let retry_config = {
        let state_ref = state.borrow();
        state_ref
            .get_connection(connection_id)
            .map(|c| c.retry_config.clone())
            .unwrap_or_default()
    };

    if !retry_config.enabled {
        return;
    }

    let mut retry_state = RetryState::new(&retry_config);

    if let Some(delay) = retry_state.next_delay() {
        let msg = i18n_f(
            "Reconnecting in {} seconds…",
            &[&format!("{}", delay.as_secs())],
        );
        // Toast з countdown та кнопкою "Cancel"
        crate::toast::show_reconnect_toast(notebook, &msg, delay, move || {
            Self::reconnect_session(state, notebook, session_id, connection_id);
        });
    }
}
```

**Вплив:** `#[serde(default)]` забезпечує backward compatibility — старі файли конфігурації працюватимуть без змін.

---

### 2.2 Інтегрувати `BatchImporter`/`BatchExporter` у GUI

**Контекст:** Пакетний імпорт/експорт реалізований, але UI використовує поодинокі операції.

**Де інтегрувати:**
- `rustconn/src/dialogs/import.rs` — при імпорті з декількох файлів одночасно
- `rustconn/src/dialogs/export.rs` — при експорті вибраних з'єднань у різні формати

**Приклад для ImportDialog (multi-file import):**
```rust
// rustconn/src/dialogs/import.rs
use rustconn_core::import::BatchImporter;

fn import_multiple_files(
    files: Vec<PathBuf>,
    format: ImportFormat,
    state: SharedAppState,
    window: &adw::ApplicationWindow,
) {
    let importer = BatchImporter::new();
    let cancel_handle = importer.cancel_handle();

    let progress = crate::dialogs::ProgressDialog::new(
        window,
        &i18n("Importing connections…"),
    );
    progress.set_cancel_action(move || cancel_handle.cancel());

    let progress_clone = progress.clone();
    glib::spawn_future_local(async move {
        let result = importer.import_all(&files, format).await;
        progress_clone.close();

        match result {
            Ok(batch_result) => {
                let msg = i18n_f(
                    "Imported {} connections, {} skipped",
                    &[
                        &batch_result.imported.to_string(),
                        &batch_result.skipped.to_string(),
                    ],
                );
                crate::toast::show_success_toast(&msg);

                // Оновити sidebar
                let mut state_mut = state.borrow_mut();
                for conn in batch_result.connections {
                    state_mut.add_connection(conn);
                }
            }
            Err(e) => {
                crate::toast::show_error_toast(&e.to_string());
            }
        }
    });
}
```

---

## Фаза 3 — Структурне покращення `MainWindow`

### 3.1 Виділити credential resolution у `window/credentials.rs`

**Контекст:** `window/mod.rs` містить 7 методів `handle_*_credentials` (~800 рядків). Виділення в окремий `impl` файл — безпечний підхід, що не змінює ownership модель GTK4.

**Методи для переносу:**
- `start_connection_with_credential_resolution` (~100 рядків)
- `handle_resolved_credentials` (~80 рядків)
- `handle_rdp_credentials` (~100 рядків)
- `handle_rdp_credentials_internal` (~90 рядків)
- `handle_vnc_credentials` (~90 рядків)
- `handle_vnc_credentials_internal` (~80 рядків)
- `start_rdp_with_password_dialog` (~20 рядків)
- `start_rdp_session_with_credentials` (~20 рядків)
- `start_vnc_with_password_dialog` (~20 рядків)

**Файл:**
```rust
// rustconn/src/window/credentials.rs
//! Credential resolution logic for MainWindow.
//!
//! Handles async vault lookups, password dialogs, and cached credentials
//! for SSH, RDP, VNC, and SPICE protocols.

use super::*;

impl MainWindow {
    /// Starts connection with async credential resolution.
    /// Acquires a busy guard (spinner) that stays alive until resolution completes.
    pub(crate) fn start_connection_with_credential_resolution(
        state: SharedAppState,
        notebook: SharedNotebook,
        split_view: SharedSplitView,
        sidebar: SharedSidebar,
        monitoring: types::SharedMonitoring,
        connection_id: Uuid,
        activity: Option<types::SharedActivityCoordinator>,
    ) {
        // ... перенести існуючий код з window/mod.rs
    }

    // ... решта методів
}
```

**У `window/mod.rs`:**
```rust
mod credentials;  // додати поруч з іншими mod declarations
```

---

### 3.2 Виділити session lifecycle у `window/session_lifecycle.rs`

**Методи для переносу (~750 рядків):**
- `setup_session_logging`
- `setup_activity_monitoring`
- `deliver_activity_notification`
- `setup_child_exited_handler`
- `setup_logging_handlers`

```rust
// rustconn/src/window/session_lifecycle.rs
//! Session lifecycle: logging, activity monitoring, child process cleanup.

use super::*;

impl MainWindow {
    /// Sets up session logging for a terminal session.
    pub fn setup_session_logging(
        state: &SharedAppState,
        notebook: &SharedNotebook,
        session_id: Uuid,
        connection: &rustconn_core::Connection,
    ) {
        // ... перенести існуючий код
    }

    // ... решта методів
}
```

**Результат після фази 3:**
- `window/mod.rs`: 5 395 → ~3 800 рядків (−30%)
- Нові файли: `credentials.rs` (~600 рядків), `session_lifecycle.rs` (~750 рядків)
- Ownership модель не змінюється — все залишається `impl MainWindow`

---

## Фаза 4 — Очищення

### 4.1 Додати `#[cfg(test)]` до зарезервованих тестових функцій

```rust
// rustconn-core/tests/properties/key_sequence_tests.rs
#[allow(dead_code)] // Reserved for future property tests
fn arb_key_sequence() -> impl Strategy<Value = KeySequence> {
    prop::collection::vec(arb_key_element(), 0..10)
        .prop_map(|elements| KeySequence::from_elements(elements))
}
```

Ця функція вже знаходиться у тестовому файлі, тому `#[cfg(test)]` не потрібен. Залишити як є — вона не потрапляє в release binary.

### 4.2 Документувати `RetryConfig` як стабільний API

Після інтеграції (Фаза 2.1) додати doc-examples:
```rust
/// Connection retry configuration with exponential backoff.
///
/// # Examples
///
/// ```
/// use rustconn_core::connection::RetryConfig;
///
/// // Default: 3 attempts, 1s initial delay, 2x backoff
/// let config = RetryConfig::default();
/// assert_eq!(config.max_attempts, 3);
///
/// // Aggressive: 5 attempts, 500ms initial, 1.5x backoff
/// let config = RetryConfig::aggressive();
/// assert_eq!(config.max_attempts, 5);
/// ```
pub struct RetryConfig { /* ... */ }
```

---

## Порядок виконання та оцінка ризику

| Фаза | Задача | Ризик | Час | Залежності |
|------|--------|-------|-----|------------|
| 1.1 | Fix `expect()` в csv_import | 🟢 Мінімальний | 15 хв | — |
| 1.2 | Розбити `performance/mod.rs` | 🟢 Мінімальний | 2 год | — |
| 1.3 | Розбити `cli_download.rs` | 🟢 Мінімальний | 2 год | — |
| 3.1 | Виділити `credentials.rs` | 🟢 Мінімальний | 1 год | — |
| 3.2 | Виділити `session_lifecycle.rs` | 🟢 Мінімальний | 1 год | 3.1 |
| 2.1 | Інтегрувати RetryConfig | 🟡 Середній | 4 год | Міграція серіалізації |
| 2.2 | Інтегрувати BatchImporter | 🟡 Середній | 3 год | — |
| 4.x | Очищення та документація | 🟢 Мінімальний | 30 хв | Після фаз 2-3 |

**Загальний час:** ~14 годин розробки

---

## Критерії завершення

- [ ] `cargo fmt --all` — без змін
- [ ] `cargo clippy --all-targets` — 0 warnings
- [ ] `cargo test --workspace` — всі тести проходять
- [ ] Жодних нових `#[allow(clippy::too_many_lines)]`
- [ ] `window/mod.rs` < 4 000 рядків (було 5 395)
- [ ] `performance/mod.rs` замінений на директорію з підмодулями
- [ ] `cli_download.rs` замінений на директорію з підмодулями
- [ ] `csv_import.rs` не містить `expect()` на user data

---

## Що НЕ робити (висновки з peer review)

1. **НЕ видаляти `#[allow(dead_code)]` у GUI-крейті** — це стандартний GTK4/Rust патерн для утримання віджетів живими
2. **НЕ створювати окремі структури** (`CredentialResolver`, `SessionManager`) — GTK4 ownership model (`Rc<RefCell<>>`) зробить це гіршим за поточний стан
3. **НЕ видаляти `BatchImporter`/`BatchExporter`** — це підготовлений API з повним тестовим покриттям
4. **НЕ видаляти `ActivityMonitorConfig`** — активно використовується через крос-крейтові залежності
5. **НЕ довіряти KiroGraph `dead_code`** для multi-crate workspace без ручної верифікації `grep`
