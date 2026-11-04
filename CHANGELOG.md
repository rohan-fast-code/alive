# Changelog

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
