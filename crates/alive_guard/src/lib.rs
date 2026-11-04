//! Guard logic for Alive (spec section 5.2). The guard is a tiny separate
//! process. It waits on the main Alive process handle with no polling, and
//! if the main process ends without writing a clean shutdown marker, the
//! guard restores the power settings from the snapshot and then exits.

use alive_core::models::PowerValues;
use alive_core::snapshot;
use alive_core::traits::PowerSettings;
use alive_core::CoreResult;
use std::path::Path;

/// The guard restores only when Alive left a dirty marker behind and did
/// not shut down cleanly.
pub fn should_restore(data_dir: &Path) -> bool {
    snapshot::is_dirty(data_dir) && !snapshot::has_clean_shutdown_marker(data_dir)
}

/// Startup self heal uses the same rule: a leftover dirty marker with no
/// clean shutdown marker means a previous run did not restore.
pub fn startup_needs_restore(data_dir: &Path) -> bool {
    should_restore(data_dir)
}

/// Restore power settings from the snapshot in `data_dir` if needed.
/// Returns true when a restore actually ran. A missing or corrupt snapshot
/// falls back to the documented Windows defaults, as the spec requires.
/// Restoring also clears the dirty marker, so running it twice is harmless.
pub fn restore_if_needed(data_dir: &Path, power: &mut dyn PowerSettings) -> CoreResult<bool> {
    if !should_restore(data_dir) {
        return Ok(false);
    }
    let values = match snapshot::load_snapshot(data_dir) {
        Ok(snap) => snap.values,
        Err(_) => PowerValues::windows_fallback_defaults(),
    };
    power.restore_all(&values)?;
    snapshot::clear_dirty(data_dir)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alive_core::mocks::MockSystem;
    use alive_core::models::Snapshot;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("alive_guard_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn original() -> PowerValues {
        PowerValues {
            lid_ac: 1,
            lid_dc: 1,
            standby_ac_secs: 1800,
            standby_dc_secs: 900,
            hibernate_ac_secs: 0,
            hibernate_dc_secs: 0,
            unattended_ac_secs: 7200,
            unattended_dc_secs: 7200,
        }
    }

    #[test]
    fn guard_restores_after_unclean_exit_and_is_idempotent() {
        // Covers the core of TC18 and TC20 at logic level: settings were
        // changed, the main process died with no clean marker, the guard
        // restores the exact snapshot values.
        let dir = temp_dir("unclean");
        let orig = original();
        snapshot::save_snapshot(&dir, &Snapshot::new(orig, false)).unwrap();
        let mut mock = MockSystem::new(PowerValues::alive_values());
        assert!(should_restore(&dir));
        assert!(restore_if_needed(&dir, &mut mock).unwrap());
        assert_eq!(mock.values, orig);
        assert!(!snapshot::is_dirty(&dir));
        // Second run does nothing and is harmless.
        assert!(!restore_if_needed(&dir, &mut mock).unwrap());
        assert_eq!(mock.values, orig);
    }

    #[test]
    fn guard_does_nothing_after_a_clean_shutdown() {
        let dir = temp_dir("clean");
        let orig = original();
        snapshot::save_snapshot(&dir, &Snapshot::new(orig, false)).unwrap();
        snapshot::mark_clean_shutdown(&dir).unwrap();
        let mut mock = MockSystem::new(PowerValues::alive_values());
        assert!(!should_restore(&dir));
        assert!(!restore_if_needed(&dir, &mut mock).unwrap());
        assert_eq!(mock.values, PowerValues::alive_values());
    }

    #[test]
    fn guard_falls_back_to_defaults_when_snapshot_is_corrupt() {
        let dir = temp_dir("corrupt");
        std::fs::write(dir.join("snapshot.json"), b"garbage").unwrap();
        std::fs::write(dir.join("dirty"), b"1").unwrap();
        let mut mock = MockSystem::new(PowerValues::alive_values());
        assert!(restore_if_needed(&dir, &mut mock).unwrap());
        assert_eq!(mock.values, PowerValues::windows_fallback_defaults());
    }
}
