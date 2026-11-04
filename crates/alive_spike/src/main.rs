//! Phase 0 hardware spike for Alive (spec section 15).
//! A throwaway console program with one job: prove or disprove the design on
//! a real Windows laptop before the full app is built around it.
//!
//! Flow on Windows:
//!   1. Read the current power values and save a snapshot plus dirty marker.
//!   2. Apply lid action Do nothing for AC and DC, and never timeouts.
//!   3. Hold the keep awake state.
//!   4. Print the three Phase 0 questions so the tester records answers.
//!   5. Wait until the tester presses Enter, then restore everything.
//!
//! On any other operating system it prints the same questions, reports that
//! the power steps are unsupported here, and exits with code 2. It never
//! pretends a hardware test happened when it did not.

use alive_core::snapshot;
use alive_core::traits::{KeepAwake, PowerSettings};
use std::io::Read as _;
use std::process::ExitCode;

const QUESTIONS: &str = "Phase 0 questions to answer in writing in docs/phase0.md\n\
1. Did applying the lid action and timeouts need elevation on a standard user account?\n\
2. Did the demo agent keep running on battery with the lid closed on a Modern Standby machine?\n\
3. What was the battery percent drop over the closed lid run, compared with the baseline?\n\
Also record: powercfg /a output, whether the machine is S0 or S3, the laptop model, and the Windows build.";

fn main() -> ExitCode {
    println!("Alive Phase 0 spike");
    println!("{QUESTIONS}");

    let data_dir = std::env::temp_dir().join("alive_spike_data");
    let mut backend = alive_win::PowercfgBackend;
    let mut hold = alive_win::KeepAwakeHold::new();

    let original = match backend.read_values() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("spike: reading power values failed here: {e}");
            eprintln!("spike: no power setting was changed on this machine");
            return ExitCode::from(2);
        }
    };
    println!("spike: original values read: {original:?}");

    let snap = alive_core::Snapshot::new(original, false);
    if let Err(e) = snapshot::save_snapshot(&data_dir, &snap) {
        eprintln!("spike: snapshot failed: {e}");
        return ExitCode::from(1);
    }

    let applied = backend
        .apply_lid_alive()
        .and_then(|_| backend.apply_timeouts_alive());
    if let Err(e) = applied {
        eprintln!("spike: applying Alive values failed: {e}");
        let _ = backend.restore_all(&original);
        let _ = snapshot::clear_dirty(&data_dir);
        return ExitCode::from(1);
    }
    if let Err(e) = hold.acquire() {
        eprintln!("spike: keep awake hold failed: {e}");
        let _ = backend.restore_all(&original);
        let _ = snapshot::clear_dirty(&data_dir);
        return ExitCode::from(1);
    }

    println!("spike: Alive values are applied and the hold is active");
    println!("spike: close the lid now, run the demo agent, then open the lid and press Enter to restore");
    let mut byte = [0u8; 1];
    let _ = std::io::stdin().read(&mut byte);

    let _ = hold.release();
    match backend.restore_all(&original) {
        Ok(()) => {
            let _ = snapshot::clear_dirty(&data_dir);
            println!("spike: original values were restored: {original:?}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("spike: restore failed: {e}");
            ExitCode::from(1)
        }
    }
}
