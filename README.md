# Alive

Alive is a Windows desktop application that keeps your AI agents running
when you close the laptop lid, and restores every power setting it changed
when you switch it OFF.

Switch Alive ON from the system tray or the window and the laptop keeps
working with the lid closed: cloud agents keep calling their APIs and
local agents keep running on the machine. Switch Alive OFF and the laptop
behaves exactly like a normal laptop again, because Alive saved your
original power settings before it changed anything and a separate guard
process restores them even if Alive is killed.

## Why a settings change alone is not enough, and an app alone is not enough

Three separate things stop an agent when the lid closes, and Alive handles
all three together:

1. The lid close action. Windows sleeps when the lid closes, and a keep
   awake request cannot override that user action. Alive sets the lid
   close action to Do nothing for both plugged in and battery, and
   restores the original values on OFF.
2. The idle sleep and hibernate timers. Alive sets them to never while
   ON, and holds a system required keep awake request on one dedicated
   thread. The display is never kept on, so the screen still turns off.
3. Modern Standby behaviour on battery. On those machines a power
   request alone is not reliable, which is why the timeouts are set as
   well, and why the battery policy pauses heavy local agents, throttles
   background work, and turns Alive OFF at a critical battery level.

## Modes

1. Strict (the default): keep everything alive regardless of power
   source, subject only to the critical battery cutoff.
2. Smart: on battery, heavy local agents pause below 30 percent and
   Alive turns OFF below 15 percent, with 5 percent of hysteresis so the
   reading cannot flap around a threshold. Battery saver pauses heavy
   agents at any percent. Cloud agents keep running down to the cutoff.
3. Plugged in only: Alive is fully ON only on mains power. On battery it
   restores normal behaviour and shows Waiting for power.

Every threshold is configurable on the Power Policy page, which also
shows exactly what Alive will change, with the original value and the
applied value side by side.

## What is in this repository

1. `crates/alive_core`: the pure Rust logic. State machine, policy
   engine, atomic snapshot with crash safety markers, config validation,
   and restart backoff. Fully unit tested, including property tests.
2. `crates/alive_win`: the Windows layer behind traits. powercfg output
   parsers, a powercfg based power settings backend, and the keep awake
   thread that calls SetThreadExecutionState.
3. `crates/alive_guard`: the tiny sidecar that watches the main process
   and restores power settings if Alive dies without cleaning up.
4. `crates/alive_spike`: the Phase 0 hardware spike console program.
5. `src-tauri`: the Tauri 2 application shell. Tray, window, and the
   full IPC command surface from the design spec.
6. `src`: the React and TypeScript frontend. Six pages: Dashboard,
   Agents, Power Policy, Activity, Settings, and About and Diagnostics.
7. `tests/hardware`: the demo agents and the manual hardware test
   sheets.
8. `docs`: the architecture in one page, the Phase 0 report, the test
   report with real results, and the threat model.

## Build and install

You need a Windows 10 or Windows 11 machine with the Rust stable MSVC
toolchain, Visual Studio Build Tools with the C++ workload and a Windows
SDK, the WebView2 runtime, and Node.js LTS.

```
npm install
npm test
cargo test --workspace
npm run tauri build
```

The last command produces the NSIS installer and the MSI in the Tauri
target release bundle folder. The installer installs per user with no
administrator rights, adds a start menu shortcut, offers an optional
Start with Windows checkbox, and its uninstall path runs the same
restore routine as the app, so an uninstall cannot leave changed power
settings behind. That uninstall hook is packaged on Windows in Phase 5;
see the known limits below.

## Signing

Release builds can be signed through environment variables in CI, using
signtool or Azure Trusted Signing with your own certificate. Unsigned
builds work for local testing, and Windows SmartScreen will warn about
them. That warning is expected and is stated plainly here rather than
hidden.

## Privacy

Alive sends nothing over the network by itself. There are no accounts,
no analytics, and no telemetry. Agent definitions run programs on your
own machine, which is why the Agents page shows a clear warning and the
full command before anything is saved. Secrets in agent environment
values are masked in logs, in the user interface, and in diagnostics
exports.

## Testing

Real results, not claims, live in `docs/test-report.md`. In short for
this build:

1. Rust: 24 tests pass across the workspace, including TC01 to TC15 from
   the design spec, run with cargo test on the build machine.
2. Frontend: 5 Vitest tests pass, the type check is clean, and the
   production build succeeds.
3. Rust quality gates: cargo fmt check and cargo clippy with warnings
   denied both pass.
4. Not yet run: the Windows integration tests and the manual hardware
   tests TC16 to TC42. They need a real Windows laptop, and
   `docs/phase0.md` lists the three Phase 0 questions that are still
   open for exactly that reason.

## Known limits in this build

1. Agent process supervision (Job Objects, per agent rotating logs,
   EcoQoS throttling) is the Phase 3 layer and is built on a Windows
   machine. The Tauri shell exposes the complete command surface now;
   the three process control commands return a clear Phase 3 message
   instead of pretending a process started. Agent definitions can
   already be created, validated, and saved.
2. The powrprof API backend and the PowerCreateRequest hold with the
   readable reason text are the next Windows step in alive_win. The
   powercfg backend in this build is the documented fallback and applies
   the same values through the same trait.
3. The guard sidecar restore logic and its process wait are written;
   packaging it as a Tauri external binary and testing a real kill on
   Windows (TC18) is Phase 5 and Phase 6 work.
4. Smart idle wake timers (design spec 6.4) are not shipped. The spec
   says to ship Strict only if they cannot be made reliable on real
   hardware, and that hardware check has not happened yet.

## Safety

Never carry a laptop in a bag while Alive is ON and the lid is closed.
There is no airflow in a bag and the heat can damage the machine and its
battery. Run closed lid work on a hard surface with the vents clear, and
prefer mains power for long runs.

## License

MIT. See the LICENSE file.
