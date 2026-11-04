//! alive_guard executable. Arguments:
//!   alive_guard --pid <main process id> --data-dir <app data folder>
//! On Windows it waits on the process handle with WaitForSingleObject and
//! no polling, as the spec requires. On other systems it reports that it is
//! unsupported and exits, because Alive is a Windows application and a guard
//! that pretends to work would be worse than one that says so plainly.

use std::path::PathBuf;
use std::process::ExitCode;

fn parse_args() -> (Option<u32>, Option<PathBuf>) {
    let mut args = std::env::args().skip(1);
    let mut pid = None;
    let mut data_dir = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pid" => {
                pid = args.next().and_then(|v| v.parse::<u32>().ok());
            }
            "--data-dir" => {
                data_dir = args.next().map(PathBuf::from);
            }
            _ => {}
        }
    }
    (pid, data_dir)
}

#[cfg(windows)]
mod win_wait {
    // SAFETY: OpenProcess, WaitForSingleObject and CloseHandle are the
    // documented kernel32 process calls. The handle returned by OpenProcess
    // is checked for null, waited on once with an infinite timeout (a handle
    // wait, not a polling loop), and closed exactly once before returning.
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> isize;
        fn WaitForSingleObject(handle: isize, milliseconds: u32) -> u32;
        fn CloseHandle(handle: isize) -> i32;
    }
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const INFINITE: u32 = 0xFFFF_FFFF;

    pub fn wait_for_process(pid: u32) -> bool {
        unsafe {
            let handle = OpenProcess(SYNCHRONIZE, 0, pid);
            if handle == 0 {
                // The process is already gone.
                return true;
            }
            let _ = WaitForSingleObject(handle, INFINITE);
            let _ = CloseHandle(handle);
            true
        }
    }
}

fn main() -> ExitCode {
    let (pid, data_dir) = parse_args();
    let Some(data_dir) = data_dir else {
        eprintln!("alive_guard: a data dir argument is required");
        return ExitCode::from(2);
    };

    #[cfg(windows)]
    let code = {
        if let Some(pid) = pid {
            win_wait::wait_for_process(pid);
        }
        let mut backend = alive_win::PowercfgBackend;
        match alive_guard::restore_if_needed(&data_dir, &mut backend) {
            Ok(restored) => {
                if restored {
                    println!("alive_guard: power settings were restored from the snapshot");
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("alive_guard: restore failed: {e}");
                ExitCode::from(1)
            }
        }
    };

    #[cfg(not(windows))]
    let code = {
        let _ = pid;
        let _ = &data_dir;
        eprintln!("alive_guard: process waiting and power restore are Windows only");
        ExitCode::from(2)
    };

    code
}
