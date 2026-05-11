# RustConn — План рефакторингу та очищення коду

Дата: 2026-05-11
Статус: **ЗАВЕРШЕНО** (6/6 тасків виконано)

---

## Резюме

Аналіз через KiroGraph виявив кілька категорій проблем:
1. ~~Мертвий код (написаний, але ніколи не підключений до GUI/CLI)~~ — **ВИКОНАНО**
2. ~~Дублювання логіки (core має реалізацію, GUI має свою паралельну)~~ — **ВИКОНАНО**
3. ~~Legacy-шар у split view (подвійне відстеження стану)~~ — **ВИКОНАНО** (публічний API видалено)
4. ~~Файли-гіганти, що потребують декомпозиції~~ — **ВИКОНАНО**

Циклічних залежностей **немає**. Архітектура 3-crate дотримується коректно.

---

## План реалізації (пріоритезований)

| # | Таск | Категорія | Складність | Ризик | Статус |
|---|---|---|---|---|---|
| 1 | ~~Видалити `ConnectionFallback` модуль~~ | B1 | Низька | Мінімальний | ✅ **v0.13.11** |
| 2 | ~~Прибрати `VirtualScrollConfig` з re-export~~ | B2 | Мінімальна | Мінімальний | ✅ **v0.13.11** |
| 3 | ~~Підключити `TaskExecutor` до GUI~~ | A1 | Середня | Середній | ✅ **v0.13.11** |
| 4 | ~~Інтегрувати `ExpectEngine` в `AutomationSession`~~ | A2 | Середня | Середній | ✅ **v0.13.12** |
| 5 | ~~Видалити legacy UUID-шар з SplitView~~ | C1+C2 | Висока | Високий | ✅ **v0.13.12** |
| 6 | ~~Декомпозиція ConnectionDialog~~ | D1 | Висока | Низький | ✅ **v0.13.11** |

---

## Виконані таски

### ✅ Таск 1: Видалити ConnectionFallback (v0.13.11)

Виконано: видалено `rustconn-core/src/connection/fallback.rs`, прибрано `pub mod fallback` та re-export з `lib.rs`. Модуль мав 0 callers за межами власних тестів.

### ✅ Таск 2: Прибрати VirtualScrollConfig з re-export (v0.13.11)

Виконано: прибрано `VirtualScrollConfig` з `pub use connection::{...}` в `lib.rs`. Залишено доступним всередині крейту для тестів.

### ✅ Таск 3: Підключити TaskExecutor до GUI (v0.13.11)

Виконано: замінено inline `std::process::Command::new("sh").arg("-c")` на виклики `TaskExecutor` через `with_runtime(|rt| rt.block_on(...))` в обох місцях:
- Pre-connect task (~рядок 3558)
- Post-disconnect task (~рядок 4047)

Тепер працюють:
- ✅ `timeout_ms` — команда переривається по таймауту
- ✅ `condition` (first/last in folder) — перевіряється через `FolderConnectionTracker`
- ✅ Variable substitution — `${var}` резолвиться з global variables
- ✅ Env sanitization — `BW_SESSION`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN` видаляються

### ✅ Таск 4: Інтегрувати ExpectEngine в AutomationSession (v0.13.12)

Виконано:
1. В `rustconn-core/src/automation/expect.rs`:
   - Додано `match_line(&str) -> Option<&CompiledRule>` — матчить з trimming та priority
   - Додано `remove_by_id(Uuid) -> bool` — видаляє без помилки якщо не знайдено
   - Додано `remove_expired(Instant, Instant) -> usize` — видаляє по спільному created_at
   - Додано `remove_expired_individual(Instant, &HashMap<Uuid, Instant>) -> usize` — per-rule timestamps
   - Додано `use std::time::Instant` для timeout API
2. В `rustconn/src/automation.rs`:
   - Видалено `struct Trigger` (замінений на `ExpectRule` + `ExpectEngine`)
   - `AutomationState` тепер містить `ExpectEngine` + `HashMap<Uuid, Instant>` замість `Vec<Trigger>` + `matched_patterns`
   - `AutomationSession::new()` приймає `Vec<ExpectRule>` замість `Vec<Trigger>`
   - `check_terminal_content()` використовує `engine.match_line()` замість ручного loop
   - One-shot → `engine.remove_by_id(matched_rule.id)`
   - Timeout → `engine.remove_expired_individual(now, &created_at)`
   - Додано `prepare_rules_from_config()` — variable substitution + validation
3. В `rustconn/src/terminal/mod.rs`:
   - Замінено створення `Vec<Trigger>` на `prepare_rules_from_config()` + `AutomationSession::new(rules)`
   - Видалено `use regex::Regex` (більше не потрібен)

**Виграш:** priority sorting, duplicate ID check, pattern validation, testability без GTK.

### ✅ Таск 5: Видалити legacy UUID-шар з SplitView (v0.13.12)

Виконано (публічний API):
1. `get_pane_session()` переписано — делегує до `adapter.get_panel_session(panel_id)` через UUID→PanelId mapping замість сканування `Vec<TerminalPane>`
2. `get_pane_color()` переписано — повертає `container_color` напряму
3. Click handlers в `window/mod.rs` та `split_view_actions.rs` — замінено `panes_ref_clone()` + manual lookup на `get_pane_session()`
4. Drop target callback — замінено `panes_rc` на `container_color_rc`
5. Видалено з публічного API: `panes_ref()`, `panes_ref_clone()`
6. `TerminalPane` → `pub(crate)` visibility, видалено з re-exports в `mod.rs`

**Залишилось (внутрішнє):** `panes` ще використовується як внутрішній кеш в деяких методах bridge.rs (split_with_close_callback, close_pane, reset). Це безпечно — зовнішній код більше не залежить від нього.

### ✅ Таск 6: Декомпозиція ConnectionDialog (v0.13.11)

Виконано: виділено `create_rdp_options` (640 рядків), `create_vnc_options` (310 рядків), `create_spice_options` (290 рядків), `create_zerotrust_options` (480 рядків) з монолітного `dialog.rs` у відповідні протокольні модулі. Видалено dead-code placeholder реалізації та замінено актуальним кодом.

Результат: `dialog.rs` 8746 → 6968 рядків (−20%). Всі протокольні модулі тепер слідують єдиному патерну (`ssh.rs`, `rdp.rs`, `vnc.rs`, `spice.rs`, `telnet.rs`, `serial.rs`, `kubernetes.rs`, `zerotrust.rs`).

---

## Що НЕ треба чіпати

- `BusyStack` — активно використовується (header bar spinner)
- `LazyGroupLoader` — використовується в sidebar
- `SelectionState` — використовується в sidebar
- `ClusterManager`/`ClusterSession` — повністю підключені
- `KeybindingSettings` — повністю підключені
- `RetryConfig`/`RetryState` — re-exported, використовуються в тестах, готові до підключення
- `ActivityMonitor*` — повністю підключені до GUI
- `AutomationTemplate`/`builtin_templates()` — використовуються в діалогах
- `ConnectionTask` struct — використовується (executor тепер підключений ✅)
