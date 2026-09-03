//! Activity monitor data models for terminal output detection.
//!
//! Global defaults live in `AppSettings.activity_monitor` and control defaults.
//! Per-connection overrides use `ActivityMonitorConfig` on the `Connection` struct.

use serde::{Deserialize, Serialize};

/// Monitoring mode for terminal output detection.
///
/// - `Off`: No monitoring (default)
/// - `Activity`: Notify when new output appears after a quiet period
/// - `Silence`: Notify when no output occurs for a configurable duration
/// - `Command`: Notify when the remote shell reports a command finished
///
/// The first three are timing heuristics over raw output. `Command` is the only
/// one driven by an actual event: VTE's `vte.shell.postexec` termprop, which the
/// remote shell sets through OSC 133 and which carries the exit code. It therefore
/// needs two things the others do not — VTE 0.78 or newer on the build host (the
/// GUI's `vte-0-78` feature) and shell integration sourced on the *remote* side.
/// This enum is data in a headless crate and carries no feature gate, so that a
/// `settings.toml` written by a build that has the feature still loads in one that
/// does not; there the mode simply never fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MonitorMode {
    /// No monitoring (default)
    #[default]
    Off,
    /// Notify on new output after quiet period
    Activity,
    /// Notify on absence of output after timeout
    Silence,
    /// Notify when the remote shell reports a finished command (OSC 133)
    Command,
}

impl MonitorMode {
    /// Cycles to the next mode: Off → Activity → Silence → Command → Off
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Off => Self::Activity,
            Self::Activity => Self::Silence,
            Self::Silence => Self::Command,
            Self::Command => Self::Off,
        }
    }

    /// Returns every mode in picker order, so no call site writes its own list.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Off, Self::Activity, Self::Silence, Self::Command]
    }

    /// Reports whether this mode fires from timing rather than from an event.
    ///
    /// `Activity` and `Silence` need the quiet-period and silence-timeout values;
    /// `Off` and `Command` ignore them, so a picker can hide those controls and the
    /// coordinator can skip arming a timer.
    #[must_use]
    pub const fn uses_timers(self) -> bool {
        matches!(self, Self::Activity | Self::Silence)
    }

    /// Returns the GTK icon name for this mode.
    #[must_use]
    pub const fn icon_name(self) -> &'static str {
        match self {
            Self::Off => "action-unavailable-symbolic",
            Self::Activity => "dialog-information-symbolic",
            Self::Silence => "dialog-warning-symbolic",
            Self::Command => "process-stop-symbolic",
        }
    }

    /// Returns the human-readable display name for this mode.
    ///
    /// Untranslated by convention — callers in the GUI wrap it in `i18n()`.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Activity => "Activity",
            Self::Silence => "Silence",
            Self::Command => "Command finished",
        }
    }
}

/// Default quiet period in seconds for activity monitoring.
const DEFAULT_QUIET_PERIOD_SECS: u32 = 10;

/// Default silence timeout in seconds for silence monitoring.
const DEFAULT_SILENCE_TIMEOUT_SECS: u32 = 30;

/// Minimum quiet period / silence timeout in seconds.
const MIN_PERIOD_SECS: u32 = 1;

/// Maximum quiet period in seconds.
const MAX_QUIET_PERIOD_SECS: u32 = 300;

/// Maximum silence timeout in seconds.
const MAX_SILENCE_TIMEOUT_SECS: u32 = 600;

/// Global activity monitor defaults stored in `AppSettings.activity_monitor`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityMonitorDefaults {
    /// Default monitoring mode
    #[serde(default)]
    pub mode: MonitorMode,
    /// Default quiet period in seconds (1–300, default: 10)
    #[serde(default = "default_quiet_period_secs")]
    pub quiet_period_secs: u32,
    /// Default silence timeout in seconds (1–600, default: 30)
    #[serde(default = "default_silence_timeout_secs")]
    pub silence_timeout_secs: u32,
}

const fn default_quiet_period_secs() -> u32 {
    DEFAULT_QUIET_PERIOD_SECS
}

const fn default_silence_timeout_secs() -> u32 {
    DEFAULT_SILENCE_TIMEOUT_SECS
}

impl Default for ActivityMonitorDefaults {
    fn default() -> Self {
        Self {
            mode: MonitorMode::Off,
            quiet_period_secs: DEFAULT_QUIET_PERIOD_SECS,
            silence_timeout_secs: DEFAULT_SILENCE_TIMEOUT_SECS,
        }
    }
}

impl ActivityMonitorDefaults {
    /// Returns the quiet period clamped to the valid range (1–300 seconds).
    #[must_use]
    pub const fn effective_quiet_period(&self) -> u32 {
        clamp_u32(
            self.quiet_period_secs,
            MIN_PERIOD_SECS,
            MAX_QUIET_PERIOD_SECS,
        )
    }

    /// Returns the silence timeout clamped to the valid range (1–600 seconds).
    #[must_use]
    pub const fn effective_silence_timeout(&self) -> u32 {
        clamp_u32(
            self.silence_timeout_secs,
            MIN_PERIOD_SECS,
            MAX_SILENCE_TIMEOUT_SECS,
        )
    }
}

/// Per-connection activity monitor override stored on `Connection`.
///
/// When `None` on a connection, the global `ActivityMonitorDefaults` apply.
/// When `Some`, these values override the global defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityMonitorConfig {
    /// Override the global monitoring mode for this connection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<MonitorMode>,
    /// Override the quiet period for this connection (seconds)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quiet_period_secs: Option<u32>,
    /// Override the silence timeout for this connection (seconds)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_timeout_secs: Option<u32>,
}

impl ActivityMonitorConfig {
    /// Returns the effective mode, falling back to the global default.
    #[must_use]
    pub fn effective_mode(&self, global: &ActivityMonitorDefaults) -> MonitorMode {
        self.mode.unwrap_or(global.mode)
    }

    /// Returns the effective quiet period, falling back to the global default
    /// and clamping to the valid range (1–300 seconds).
    #[must_use]
    pub fn effective_quiet_period(&self, global: &ActivityMonitorDefaults) -> u32 {
        let secs = self
            .quiet_period_secs
            .unwrap_or_else(|| global.effective_quiet_period());
        clamp_u32(secs, MIN_PERIOD_SECS, MAX_QUIET_PERIOD_SECS)
    }

    /// Returns the effective silence timeout, falling back to the global default
    /// and clamping to the valid range (1–600 seconds).
    #[must_use]
    pub fn effective_silence_timeout(&self, global: &ActivityMonitorDefaults) -> u32 {
        let secs = self
            .silence_timeout_secs
            .unwrap_or_else(|| global.effective_silence_timeout());
        clamp_u32(secs, MIN_PERIOD_SECS, MAX_SILENCE_TIMEOUT_SECS)
    }
}

/// Const-compatible u32 clamp (stable `u32::clamp` is not const).
const fn clamp_u32(val: u32, min: u32, max: u32) -> u32 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_cycling() {
        assert_eq!(MonitorMode::Off.next(), MonitorMode::Activity);
        assert_eq!(MonitorMode::Activity.next(), MonitorMode::Silence);
        assert_eq!(MonitorMode::Silence.next(), MonitorMode::Command);
        assert_eq!(MonitorMode::Command.next(), MonitorMode::Off);
    }

    #[test]
    fn cycling_visits_every_mode_and_returns_to_the_start() {
        // The per-tab Monitor menu cycles with `next()`, so a mode missing from the
        // cycle would be unreachable from the UI even though it exists in settings.
        let mut seen = Vec::new();
        let mut mode = MonitorMode::Off;
        for _ in 0..MonitorMode::all().len() {
            seen.push(mode);
            mode = mode.next();
        }
        assert_eq!(mode, MonitorMode::Off, "cycle must close");
        assert_eq!(seen.len(), MonitorMode::all().len());
        for candidate in MonitorMode::all() {
            assert!(seen.contains(candidate), "{candidate:?} is unreachable");
        }
    }

    #[test]
    fn only_the_timing_modes_report_using_timers() {
        assert!(!MonitorMode::Off.uses_timers());
        assert!(MonitorMode::Activity.uses_timers());
        assert!(MonitorMode::Silence.uses_timers());
        // Command is event-driven: it must not arm a silence timer.
        assert!(!MonitorMode::Command.uses_timers());
    }

    #[test]
    fn every_mode_has_a_distinct_icon_and_name() {
        let mut names: Vec<&str> = MonitorMode::all()
            .iter()
            .map(|m| m.display_name())
            .collect();
        let before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), before, "display names must be distinct");

        let mut icons: Vec<&str> = MonitorMode::all().iter().map(|m| m.icon_name()).collect();
        let before = icons.len();
        icons.sort_unstable();
        icons.dedup();
        assert_eq!(icons.len(), before, "icon names must be distinct");
    }

    #[test]
    fn command_mode_survives_a_serde_round_trip() {
        // Written by a build that has the vte-0-78 feature, read by one that does
        // not: the value must still parse rather than reset the whole settings file.
        let json = serde_json::to_string(&MonitorMode::Command).expect("serialize");
        assert_eq!(json, "\"command\"");
        let back: MonitorMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, MonitorMode::Command);
    }

    #[test]
    fn test_mode_default_is_off() {
        assert_eq!(MonitorMode::default(), MonitorMode::Off);
    }

    #[test]
    fn test_defaults_values() {
        let d = ActivityMonitorDefaults::default();
        assert_eq!(d.mode, MonitorMode::Off);
        assert_eq!(d.quiet_period_secs, 10);
        assert_eq!(d.silence_timeout_secs, 30);
    }

    #[test]
    fn test_defaults_clamping_zero() {
        let d = ActivityMonitorDefaults {
            quiet_period_secs: 0,
            silence_timeout_secs: 0,
            ..Default::default()
        };
        assert_eq!(d.effective_quiet_period(), 1);
        assert_eq!(d.effective_silence_timeout(), 1);
    }

    #[test]
    fn test_defaults_clamping_overflow() {
        let d = ActivityMonitorDefaults {
            quiet_period_secs: 999,
            silence_timeout_secs: 999,
            ..Default::default()
        };
        assert_eq!(d.effective_quiet_period(), 300);
        assert_eq!(d.effective_silence_timeout(), 600);
    }

    #[test]
    fn test_config_fallback_to_global() {
        let global = ActivityMonitorDefaults::default();
        let config = ActivityMonitorConfig {
            mode: None,
            quiet_period_secs: None,
            silence_timeout_secs: None,
        };
        assert_eq!(config.effective_mode(&global), MonitorMode::Off);
        assert_eq!(config.effective_quiet_period(&global), 10);
        assert_eq!(config.effective_silence_timeout(&global), 30);
    }

    #[test]
    fn test_config_override() {
        let global = ActivityMonitorDefaults::default();
        let config = ActivityMonitorConfig {
            mode: Some(MonitorMode::Activity),
            quiet_period_secs: Some(20),
            silence_timeout_secs: Some(60),
        };
        assert_eq!(config.effective_mode(&global), MonitorMode::Activity);
        assert_eq!(config.effective_quiet_period(&global), 20);
        assert_eq!(config.effective_silence_timeout(&global), 60);
    }

    #[test]
    fn test_config_clamping() {
        let global = ActivityMonitorDefaults::default();
        let config = ActivityMonitorConfig {
            mode: None,
            quiet_period_secs: Some(0),
            silence_timeout_secs: Some(1000),
        };
        assert_eq!(config.effective_quiet_period(&global), 1);
        assert_eq!(config.effective_silence_timeout(&global), 600);
    }

    #[test]
    fn test_serde_roundtrip_config() {
        let config = ActivityMonitorConfig {
            mode: Some(MonitorMode::Silence),
            quiet_period_secs: Some(15),
            silence_timeout_secs: None,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: ActivityMonitorConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_defaults() {
        let defaults = ActivityMonitorDefaults {
            mode: MonitorMode::Activity,
            quiet_period_secs: 5,
            silence_timeout_secs: 120,
        };
        let json = serde_json::to_string(&defaults).expect("serialize");
        let deserialized: ActivityMonitorDefaults =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(defaults, deserialized);
    }

    #[test]
    fn test_skip_serializing_none_fields() {
        let config = ActivityMonitorConfig {
            mode: None,
            quiet_period_secs: None,
            silence_timeout_secs: None,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        assert_eq!(json, "{}");
    }
}
