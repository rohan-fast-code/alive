//! Snapshot persistence with crash safety (spec section 5.2).
//! Files in the app data folder:
//!   snapshot.json  the power values taken before any change
//!   dirty          marker that exists while Alive settings may be applied
//!   clean_shutdown marker written by a clean exit, read by the guard
//! Every write is atomic: write a temp file, flush it, then rename it over
//! the target, so a crash mid write can never leave a half written snapshot.

use crate::models::Snapshot;
use crate::{CoreError, CoreResult};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn snapshot_path(dir: &Path) -> PathBuf {
    dir.join("snapshot.json")
}
pub fn dirty_path(dir: &Path) -> PathBuf {
    dir.join("dirty")
}
pub fn clean_marker_path(dir: &Path) -> PathBuf {
    dir.join("clean_shutdown")
}

fn write_atomic(path: &Path, bytes: &[u8]) -> CoreResult<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(|e| CoreError::Io(e.to_string()))?;
        f.write_all(bytes)
            .map_err(|e| CoreError::Io(e.to_string()))?;
        f.sync_all().map_err(|e| CoreError::Io(e.to_string()))?;
    }
    fs::rename(&tmp, path).map_err(|e| CoreError::Io(e.to_string()))?;
    Ok(())
}

/// Persist the snapshot and the dirty marker, in that order. This must be
/// called before any power setting is changed.
pub fn save_snapshot(dir: &Path, snapshot: &Snapshot) -> CoreResult<()> {
    fs::create_dir_all(dir).map_err(|e| CoreError::Io(e.to_string()))?;
    let json = serde_json::to_string_pretty(snapshot).map_err(|e| CoreError::Io(e.to_string()))?;
    write_atomic(&snapshot_path(dir), json.as_bytes())?;
    write_atomic(&dirty_path(dir), b"1")?;
    // A new arming invalidates any earlier clean shutdown marker.
    let _ = fs::remove_file(clean_marker_path(dir));
    Ok(())
}

/// Load and validate a snapshot. Any problem at all is reported as
/// CorruptSnapshot so callers fall back to the Windows defaults and tell
/// the user, instead of trusting a partial file.
pub fn load_snapshot(dir: &Path) -> CoreResult<Snapshot> {
    let text = fs::read_to_string(snapshot_path(dir)).map_err(|_| CoreError::CorruptSnapshot)?;
    let snap: Snapshot = serde_json::from_str(&text).map_err(|_| CoreError::CorruptSnapshot)?;
    if !snap.is_valid() {
        return Err(CoreError::CorruptSnapshot);
    }
    Ok(snap)
}

pub fn is_dirty(dir: &Path) -> bool {
    dirty_path(dir).exists()
}

pub fn clear_dirty(dir: &Path) -> CoreResult<()> {
    let _ = fs::remove_file(dirty_path(dir));
    Ok(())
}

pub fn mark_clean_shutdown(dir: &Path) -> CoreResult<()> {
    fs::create_dir_all(dir).map_err(|e| CoreError::Io(e.to_string()))?;
    write_atomic(&clean_marker_path(dir), b"1")
}

pub fn has_clean_shutdown_marker(dir: &Path) -> bool {
    clean_marker_path(dir).exists()
}
