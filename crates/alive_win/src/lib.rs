//! alive_win: the Windows layer for Alive, behind the alive_core traits.
//!
//! What is real and tested on any OS here:
//!   * Pure parsers for powercfg query output and for powercfg /a output.
//!   * A powercfg based PowerSettings backend (the fallback the spec asks to
//!     keep behind the same trait as the powrprof API backend).
//!
//! What is Windows only and therefore compiled only on Windows:
//!   * The dedicated keep awake thread that calls SetThreadExecutionState.
//!
//! On other operating systems the keep awake type returns Unsupported, and the
//! powercfg backend fails cleanly because the powercfg program does not exist.
//! Alive itself is a Windows application; this crate existing on Linux is for
//! tests and development only.

use alive_core::models::PowerValues;
use alive_core::traits::*;
use alive_core::{CoreError, CoreResult};
use std::process::Command;

// ---------------------------------------------------------------------------
// Pure parsers
// ---------------------------------------------------------------------------

/// Parse one current power setting index out of `powercfg /query` output.
/// `ac` selects the AC line, otherwise the DC line is used. The line looks
/// like: `Current AC Power Setting Index: 0x00000001`
pub fn parse_setting_index(output: &str, ac: bool) -> Option<u32> {
    let needle = if ac {
        "Current AC Power Setting Index:"
    } else {
        "Current DC Power Setting Index:"
    };
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(needle) {
            let text = rest
                .trim()
                .trim_start_matches("0x")
                .trim_start_matches("0X");
            return u32::from_str_radix(text, 16).ok();
        }
    }
    None
}

/// Parse the lid action for AC and DC from a LIDACTION query. Query one
/// setting at a time in real code; this helper is for a combined capture.
pub fn parse_lid_values(query_ac_output: &str, query_dc_source_output: &str) -> Option<(u32, u32)> {
    Some((
        parse_setting_index(query_ac_output, true)?,
        parse_setting_index(query_dc_source_output, false)?,
    ))
}

/// True when `powercfg /a` output shows Modern Standby (S0 low power idle).
pub fn detect_modern_standby(powercfg_a_output: &str) -> bool {
    powercfg_a_output.contains("S0 Low Power Idle") || powercfg_a_output.contains("Standby (S0")
}

/// True when `powercfg /a` output shows the classic S3 sleep state.
pub fn detect_s3(powercfg_a_output: &str) -> bool {
    powercfg_a_output.contains("Standby (S3)")
}

// ---------------------------------------------------------------------------
// powercfg fallback backend
// ---------------------------------------------------------------------------

fn run_powercfg(args: &[&str]) -> CoreResult<String> {
    let out = Command::new("powercfg")
        .args(args)
        .output()
        .map_err(|e| CoreError::Unsupported(format!("powercfg could not run: {e}")))?;
    if !out.status.success() {
        return Err(CoreError::StepFailed {
            step: "powercfg".to_string(),
            message: String::from_utf8_lossy(&out.stderr).to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn query_setting(subgroup_and_setting: &[&str]) -> CoreResult<String> {
    let mut args = vec!["/query", "SCHEME_CURRENT"];
    args.extend_from_slice(subgroup_and_setting);
    run_powercfg(&args)
}

fn read_one(subgroup_and_setting: &[&str], ac: bool) -> CoreResult<u32> {
    let text = query_setting(subgroup_and_setting)?;
    parse_setting_index(&text, ac).ok_or_else(|| CoreError::StepFailed {
        step: "powercfg".to_string(),
        message: "setting index not found in powercfg output".to_string(),
    })
}

const SUB_BUTTONS: &str = "SUB_BUTTONS";
const SUB_SLEEP: &str = "SUB_SLEEP";

/// PowerSettings backed by the powercfg program. The production Windows
/// build prefers the powrprof APIs; this backend is the documented fallback
/// and is also what the Phase 0 spike and the guard use, because it has no
/// extra dependencies and works even when the main app is gone.
pub struct PowercfgBackend;

impl PowercfgBackend {
    fn set_index(&mut self, subgroup: &str, setting: &str, ac: bool, value: u32) -> CoreResult<()> {
        let verb = if ac {
            "/setacvalueindex"
        } else {
            "/setdcvalueindex"
        };
        let value_text = value.to_string();
        run_powercfg(&[verb, "SCHEME_CURRENT", subgroup, setting, &value_text])?;
        Ok(())
    }
    fn activate(&mut self) -> CoreResult<()> {
        run_powercfg(&["/setactive", "SCHEME_CURRENT"])?;
        Ok(())
    }
}

impl PowerSettings for PowercfgBackend {
    fn read_values(&mut self) -> CoreResult<PowerValues> {
        Ok(PowerValues {
            lid_ac: read_one(&[SUB_BUTTONS, "LIDACTION"], true)?,
            lid_dc: read_one(&[SUB_BUTTONS, "LIDACTION"], false)?,
            standby_ac_secs: read_one(&[SUB_SLEEP, "STANDBYIDLE"], true)?,
            standby_dc_secs: read_one(&[SUB_SLEEP, "STANDBYIDLE"], false)?,
            hibernate_ac_secs: read_one(&[SUB_SLEEP, "HIBERNATEIDLE"], true)?,
            hibernate_dc_secs: read_one(&[SUB_SLEEP, "HIBERNATEIDLE"], false)?,
            // The unattended sleep timeout uses the same subgroup with its
            // own setting name on Windows. If a machine does not expose it,
            // the read fails and the caller treats the machine as
            // unsupported rather than guessing a value.
            unattended_ac_secs: read_one(&[SUB_SLEEP, "UNATTENDSLEEP"], true)?,
            unattended_dc_secs: read_one(&[SUB_SLEEP, "UNATTENDSLEEP"], false)?,
        })
    }

    fn apply_lid_alive(&mut self) -> CoreResult<()> {
        self.set_index(SUB_BUTTONS, "LIDACTION", true, 0)?;
        self.set_index(SUB_BUTTONS, "LIDACTION", false, 0)?;
        self.activate()
    }

    fn apply_timeouts_alive(&mut self) -> CoreResult<()> {
        for setting in ["STANDBYIDLE", "HIBERNATEIDLE", "UNATTENDSLEEP"] {
            self.set_index(SUB_SLEEP, setting, true, 0)?;
            self.set_index(SUB_SLEEP, setting, false, 0)?;
        }
        self.activate()
    }

    fn restore_all(&mut self, values: &PowerValues) -> CoreResult<()> {
        self.set_index(SUB_BUTTONS, "LIDACTION", true, values.lid_ac)?;
        self.set_index(SUB_BUTTONS, "LIDACTION", false, values.lid_dc)?;
        self.set_index(SUB_SLEEP, "STANDBYIDLE", true, values.standby_ac_secs)?;
        self.set_index(SUB_SLEEP, "STANDBYIDLE", false, values.standby_dc_secs)?;
        self.set_index(SUB_SLEEP, "HIBERNATEIDLE", true, values.hibernate_ac_secs)?;
        self.set_index(SUB_SLEEP, "HIBERNATEIDLE", false, values.hibernate_dc_secs)?;
        self.set_index(SUB_SLEEP, "UNATTENDSLEEP", true, values.unattended_ac_secs)?;
        self.set_index(SUB_SLEEP, "UNATTENDSLEEP", false, values.unattended_dc_secs)?;
        self.activate()
    }
}

// ---------------------------------------------------------------------------
// Keep awake hold
// ---------------------------------------------------------------------------

/// The keep awake hold. On Windows it owns one dedicated thread that calls
/// SetThreadExecutionState with ES_CONTINUOUS and ES_SYSTEM_REQUIRED and then
/// parks on a channel, because the execution state belongs to the calling
/// thread (spec sections 3 and 5.3). The display required flag is never used,
/// so the screen can still turn off. A power request with the reason text
/// "Alive is keeping AI agents running" is the documented next step for the
/// Windows build and is noted in the architecture doc; the thread state call
/// is the part that actually holds the system awake.
pub struct KeepAwakeHold {
    held: bool,
    #[cfg(windows)]
    release_sender: Option<std::sync::mpsc::Sender<()>>,
}

impl KeepAwakeHold {
    pub fn new() -> Self {
        Self {
            held: false,
            #[cfg(windows)]
            release_sender: None,
        }
    }
}

impl Default for KeepAwakeHold {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
mod win_keep_awake {
    // SAFETY: SetThreadExecutionState is a documented kernel32 call that
    // takes a plain integer flag set and returns the previous flags. It has
    // no pointer arguments, so this declaration cannot alias or outlive
    // anything. It is only ever called from the one dedicated thread that
    // owns the hold.
    extern "system" {
        pub fn SetThreadExecutionState(es_flags: u32) -> u32;
    }
    pub const ES_CONTINUOUS: u32 = 0x8000_0000;
    pub const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
}

impl KeepAwake for KeepAwakeHold {
    #[cfg(windows)]
    fn acquire(&mut self) -> CoreResult<()> {
        if self.held {
            return Ok(());
        }
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            // SAFETY: see the declaration comment in win_keep_awake.
            unsafe {
                win_keep_awake::SetThreadExecutionState(
                    win_keep_awake::ES_CONTINUOUS | win_keep_awake::ES_SYSTEM_REQUIRED,
                );
            }
            // Park with no loop and no timer until release is signalled.
            let _ = rx.recv();
            // SAFETY: same call, clearing back to ES_CONTINUOUS alone.
            unsafe {
                win_keep_awake::SetThreadExecutionState(win_keep_awake::ES_CONTINUOUS);
            }
        });
        self.release_sender = Some(tx);
        self.held = true;
        Ok(())
    }

    #[cfg(not(windows))]
    fn acquire(&mut self) -> CoreResult<()> {
        Err(CoreError::Unsupported(
            "SetThreadExecutionState exists only on Windows".to_string(),
        ))
    }

    fn release(&mut self) -> CoreResult<()> {
        #[cfg(windows)]
        {
            if let Some(tx) = self.release_sender.take() {
                let _ = tx.send(());
            }
        }
        self.held = false;
        Ok(())
    }

    fn is_held(&self) -> bool {
        self.held
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUERY_AC: &str = "    Current AC Power Setting Index: 0x00000000\n    Current DC Power Setting Index: 0x00000001\n";

    #[test]
    fn parses_ac_and_dc_setting_indexes() {
        assert_eq!(parse_setting_index(QUERY_AC, true), Some(0));
        assert_eq!(parse_setting_index(QUERY_AC, false), Some(1));
        assert_eq!(parse_setting_index("nothing here", true), None);
    }

    #[test]
    fn parses_lid_pair() {
        assert_eq!(parse_lid_values(QUERY_AC, QUERY_AC), Some((0, 1)));
    }

    #[test]
    fn detects_modern_standby_and_s3() {
        let s0 = "Standby (S0 Low Power Idle) Network Connected\nStandby (S3) is not available";
        // The S0 helper must be true, and a plain S3 listing must not be
        // misread as Modern Standby.
        assert!(detect_modern_standby(s0));
        assert!(!detect_modern_standby("Standby (S3)\nHibernate"));
        assert!(detect_s3("Standby (S3)"));
    }

    #[test]
    fn keep_awake_is_not_held_at_start_and_release_is_safe() {
        let mut hold = KeepAwakeHold::new();
        assert!(!hold.is_held());
        // Releasing a hold that was never taken must be harmless.
        hold.release().unwrap();
        assert!(!hold.is_held());
    }

    #[cfg(not(windows))]
    #[test]
    fn keep_awake_acquire_is_unsupported_off_windows() {
        let mut hold = KeepAwakeHold::new();
        assert!(hold.acquire().is_err());
        assert!(!hold.is_held());
    }
}
