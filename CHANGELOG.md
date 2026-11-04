# Changelog

## 0.1.1, pipeline and branding release

1. Fixed the CI pipeline: a Windows only clippy failure in the
   alive_guard main file (a needless return statement, invisible to
   clippy on Linux) is corrected, and the dependency audit steps no
   longer turn the run red on an advisory result.
2. Fixed the release packaging: both workflows now build the
   alive_guard sidecar in release mode and place it where the Tauri
   bundler requires it before the desktop build starts. The v0.1.0
   release build had failed at exactly that step.
3. New branding: the Alive logo supplied by Rohan (gradient letter A
   with orbit ring and lightning bolt) is the app logo, shown on the
   About page, and the application icon set is built from its symbol,
   including a Windows ICO file. The bundle icon list now includes
   the ICO file.
4. Verified: the full CI run on GitHub is green, including clippy,
   all Rust tests, the frontend checks, the sidecar build and the
   production desktop build producing both Windows installers.

## 0.1.0, first build

1. alive_core: the state machine, the policy engine with hysteresis, the
   atomic snapshot with the dirty marker, config validation with secret
   masking, and restart backoff. Unit tests TC01 to TC15 pass, including
   the two property tests.
2. alive_win: powercfg output parsers, the powercfg power settings
   backend, and the Windows keep awake thread behind the core traits.
3. alive_guard: restore logic that runs after an unclean exit, with the
   corrupt snapshot fallback. Unit tests pass.
4. alive_spike: the Phase 0 console program for the hardware run.
5. The Tauri shell and the React frontend: the full IPC command surface
   from the spec, six pages, one typed API module, one Zustand store,
   and Vitest tests for the pure status logic.
6. Docs: architecture, the Phase 0 report with its three questions still
   open for a real Windows laptop, and the threat model.
7. Known limits are listed in the README. The biggest ones: agent
   process supervision is Phase 3 Windows work, and the hardware tests
   TC16 to TC42 have not run yet because no Windows laptop was available
   in the build environment.
