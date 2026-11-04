# Alive architecture in one page

## What Alive is

A Windows tray application with one job: while it is ON, the laptop keeps
running the user defined AI agents with the lid closed and uses as little
battery as the policy allows; while it is OFF, every power setting is
exactly what it was before, guaranteed by a snapshot and a guard process.

## Crates

1. `alive_core`: pure Rust logic with no Windows calls. The state machine
   (Off, Arming, OnActive, OnIdle, OnThrottled, Restoring, Faulted), the
   policy engine (Strict, Smart, Plugged in only, with thresholds and a
   5 percent hysteresis latch), atomic snapshot persistence with a dirty
   marker, config models with validation and secret masking, and restart
   backoff with deterministic jitter. Everything here is unit tested,
   including property tests over random toggle sequences.
2. `alive_win`: the Windows layer behind the core traits. It contains the
   pure powercfg output parsers, a powercfg based power settings backend
   (the documented fallback for the powrprof API backend), and the
   dedicated keep awake thread that calls SetThreadExecutionState with
   ES_CONTINUOUS and ES_SYSTEM_REQUIRED and parks on a channel. The
   display required flag is never used. The next Windows step is the
   powrprof API backend and a PowerCreateRequest hold with the reason
   text "Alive is keeping AI agents running", so `powercfg /requests`
   shows a readable reason (TC16 checks exactly that output).
3. `alive_guard`: a tiny sidecar. It waits on the main process handle
   with no polling, and restores settings from the snapshot when the
   main process ends without a clean shutdown marker. Its restore logic
   is unit tested, including the corrupt snapshot fallback to the
   documented Windows defaults.
4. `alive_spike`: the Phase 0 console program. See docs/phase0.md.
5. The Tauri application in `src-tauri`: tray, window, and the IPC
   commands from spec section 7, wired to `alive_core`. It is excluded
   from the Linux workspace build because Tauri needs the Windows
   WebView2 and MSVC toolchain; it builds on Windows with
   `npm run tauri build`.
6. The React frontend in `src`: one typed API module wraps every invoke
   call, state lives in one Zustand store, and the six pages from the
   spec are implemented (Dashboard, Agents, Power Policy, Activity,
   Settings, About and Diagnostics).

## The toggle, step by step

ON: snapshot and dirty marker first, then guard, then lid action Do
nothing for AC and DC, then sleep and hibernate timeouts never, then the
keep awake hold, then power event subscription, then agents. Any failure
undoes the completed steps in reverse order and ends in OFF with the
error shown.

OFF: stop agents, release the hold, restore every value from the
snapshot, clear the dirty marker, stop the guard. Running OFF twice is
harmless.

## Crash safety

The dirty marker plus snapshot is the whole guarantee. Startup checks
for a leftover dirty marker and restores before showing any UI. The
guard covers a kill while the app is running. A corrupt snapshot falls
back to the documented defaults and the user is told. The uninstaller
must run the same restore routine; that hook is part of the Phase 5
packaging work on Windows.

## What is deliberately not here yet

Agent process supervision with Job Objects, per agent rotating logs and
EcoQoS throttling is Phase 3 and needs a Windows machine to build and
test honestly. The Tauri shell already exposes the full command surface;
the three process control commands return a clear Phase 3 error instead
of pretending a process started. Smart idle wake timers (spec 6.4) are
not attempted yet for the same reason: the spec says to ship Strict only
if they cannot be made reliable on real hardware.
