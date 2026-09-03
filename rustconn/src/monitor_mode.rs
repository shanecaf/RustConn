//! Picker plumbing for [`MonitorMode`], in one place.
//!
//! The mode reaches the user through four widgets — the global default combo in
//! Preferences ▸ Monitoring, the per-connection override combo in the connection
//! editor, and the load/collect halves of each. Every one of them used to carry its
//! own hand-written index map:
//!
//! ```text
//! let mode = match combo.selected() { 1 => Activity, 2 => Silence, _ => Off };
//! ```
//!
//! Seven copies across four files, which is why adding a variant is a bug rather
//! than an edit: a copy that is missed silently maps the new mode onto `Off`, and
//! nothing fails until a user notices their setting will not stick. The list of
//! modes now comes from [`MonitorMode::all`] and the mapping from
//! [`index_of`]/[`from_index`], so a future variant is one line in `rustconn-core`.

use rustconn_core::activity_monitor::MonitorMode;

use crate::i18n::i18n;

/// Translated labels for the mode picker, in [`MonitorMode::all`] order.
#[must_use]
pub fn labels() -> Vec<String> {
    MonitorMode::all()
        .iter()
        .map(|mode| i18n(mode.display_name()))
        .collect()
}

/// The picker index showing `mode`.
///
/// Falls back to 0 (`Off`) for a mode that is somehow absent from
/// [`MonitorMode::all`], which is the safe direction: showing "Off" for something
/// unrecognised is better than showing the wrong mode as if it were selected.
#[must_use]
pub fn index_of(mode: MonitorMode) -> u32 {
    let position = MonitorMode::all().iter().position(|m| *m == mode);
    u32::try_from(position.unwrap_or(0)).unwrap_or(0)
}

/// The mode at picker index `index`, or `Off` if the index is out of range.
#[must_use]
pub fn from_index(index: u32) -> MonitorMode {
    usize::try_from(index)
        .ok()
        .and_then(|i| MonitorMode::all().get(i).copied())
        .unwrap_or_default()
}

/// One sentence for the mode row's subtitle, naming what Command mode needs.
///
/// Command mode is the only one that depends on the far end: VTE learns a command
/// finished because the *remote* shell said so through OSC 133, which means `vte.sh`
/// or an equivalent has to be sourced there. Saying so in the row is the difference
/// between a setting that looks broken and one the user can act on.
#[must_use]
pub fn mode_row_subtitle() -> String {
    i18n("Command finished needs shell integration on the remote host")
}

#[cfg(test)]
mod tests {
    use rustconn_core::activity_monitor::MonitorMode;

    use super::{from_index, index_of};

    #[test]
    fn every_mode_round_trips_through_its_index() {
        for mode in MonitorMode::all() {
            assert_eq!(
                from_index(index_of(*mode)),
                *mode,
                "{mode:?} does not survive the picker"
            );
        }
    }

    #[test]
    fn off_is_first_so_an_unset_picker_means_off() {
        // Several call sites build a combo with `.selected(0)` before loading a
        // value; that default has to be the harmless mode.
        assert_eq!(from_index(0), MonitorMode::Off);
        assert_eq!(index_of(MonitorMode::Off), 0);
    }

    #[test]
    fn an_out_of_range_index_falls_back_to_off() {
        assert_eq!(from_index(u32::MAX), MonitorMode::Off);
        let past_the_end = u32::try_from(MonitorMode::all().len()).expect("small");
        assert_eq!(from_index(past_the_end), MonitorMode::Off);
    }
}
