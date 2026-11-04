# Alive test report

Build environment for this report: a Linux machine (no Windows laptop was
available), Rust stable 1.99.1, Node.js 24, npm with Vite 5 and Vitest 2.
Every result below was produced by a command that was actually run in
this build. Anything that needs Windows is listed as not run, with the
reason, instead of being described as passing.

## Unit tests, spec section 11.1 (cargo test, alive_core)

Command: `cargo test --workspace`

Result: **all workspace tests pass. 24 tests in total, 0 failed.**

alive_core integration tests in `crates/alive_core/tests/tc_tests.rs`:

1. TC01 Toggle ON emits the exact ordered sequence snapshot, guard, lid,
   timeouts, hold, subscribe, agents: PASS
2. TC02 Toggle OFF restores every setting exactly for AC and DC, and a
   second OFF is harmless: PASS
3. TC03 Fifty toggles (25 ON and OFF pairs) end in OFF with no leaked
   hold, no running guard, and the original values intact: PASS
4. TC04 Failure injected at each of the seven Arming steps, one at a
   time, undoes the completed steps, runs no later step, and ends in OFF
   with the original values intact: PASS
5. TC05 Policy on AC allows all agents at 100, 50, 28, 14 and 1 percent,
   in Smart and in Strict: PASS
6. TC06 Smart on battery at 80 percent allows all: PASS
7. TC07 Smart on battery at 28 percent pauses heavy agents: PASS
8. TC08 Battery at 14 percent gives the critical decision in Smart and
   in Strict: PASS
9. TC09 Battery oscillating between 29 and 31 percent for 20 readings
   produces at most one transition: PASS
10. TC10 Battery saver active at 95 percent pauses heavy agents: PASS
11. TC11 Plugged in only on battery waits for power, and on AC allows
    all: PASS
12. TC12 Config validation rejects a missing command, duplicate names,
    a nonexistent path, negative numbers and oversized values, and
    accepts a valid config with a real file: PASS
13. TC13 Snapshot roundtrip works through the atomic save, truncated
    and garbage snapshot files are detected as corrupt, and the
    fallback defaults have the lid action at Sleep: PASS
14. TC14 Backoff grows exponentially, stays inside the documented
    jitter band, is capped at the maximum, and the restart counter
    resets only after a stable run: PASS
15. TC15 Property tests with proptest, 200 cases each: random toggle
    sequences preserve the invariants (OFF means no hold, no guard, and
    values equal to the snapshot, and restoring is idempotent), and the
    policy never gives a critical decision on AC: PASS

alive_win unit tests: PASS, 5 tests. powercfg setting index parsing for
AC and DC, lid pair parsing, Modern Standby and S3 detection with no
misread of a plain S3 listing, and the keep awake hold starting unheld
with a safe release and an honest unsupported result off Windows.

alive_guard unit tests: PASS, 3 tests. The guard restores the exact
snapshot values after an unclean exit and is idempotent, does nothing
after a clean shutdown, and falls back to the documented defaults when
the snapshot is corrupt. These three tests are the logic core of TC18
and TC20; the full tests still need a real Windows process kill, below.

## Frontend tests and build

1. `npm test` (Vitest): PASS, 5 tests in `src/logic/status.test.ts`,
   covering the ON states, every lid action label, battery text,
   the tray tooltip text, and transition counting.
2. `npm run lint` (TypeScript with strict mode and no unchecked indexed
   access): PASS, no errors.
3. `npm run build` (type check plus the Vite production build): PASS.
   Output sizes: the main JavaScript file is 173.64 kB, 56.01 kB
   gzipped; the CSS file is 7.60 kB, 2.33 kB gzipped.

## Quality gates that ran here

1. `cargo fmt --all -- --check`: PASS
2. `cargo clippy --workspace --all-targets -- -D warnings`: PASS, no
   warnings. Three warnings found during the build (a derivable Default
   implementation, a collapsible match, and doc list formatting) were
   fixed in the code rather than allowed.
3. Demo agent syntax: both Python demo agents compile cleanly.
4. Every JSON file in the project (package, TypeScript config, Tauri
   config, capabilities) parses as valid JSON.

## Measured numbers that are available here

1. alive_guard release binary on Linux: 355 kB, well under the 1.5 MB
   target in the spec. The Windows binary size must be measured again
   on Windows; this number is an indication, not the Windows result.
2. alive_spike release binary on Linux: 460 kB.

## Not run, and why

1. TC16 to TC25, the Windows integration tests: NOT RUN. They need the
   real Windows power APIs, a real process kill, a real logoff, and the
   packaged app. The logic underneath TC18 and TC20 is unit tested
   above; the operating system level proof is Phase 2 and Phase 6
   Windows work.
2. TC26 to TC30, the WebDriver UI tests: NOT RUN. tauri-driver and the
   Edge WebDriver exist only in the Windows setup. The frontend logic
   that TC27 and TC28 depend on (validation rules and the lid and
   timeout labels) is covered by the Vitest and Rust tests above.
3. TC31 to TC42, the manual hardware tests: NOT RUN. They need a real
   laptop lid, a battery, and WiFi. The sheets and the demo agents are
   in `tests/hardware`, and the three Phase 0 questions in
   `docs/phase0.md` stay open until that run happens.
4. The Tauri production build and the installer (TC40): NOT BUILT here.
   Tauri needs the MSVC toolchain and the WebView2 runtime. The GitHub
   Actions workflow in `.github/workflows/ci.yml` runs the full build
   on a Windows runner, and the release workflow builds the installer
   and the MSI for a tagged version.
5. Efficiency budgets from spec 6.5 (idle CPU, memory with the window
   closed, window open time, port listing): NOT MEASURED here, because
   they describe the running Windows app. TC38 in the hardware sheets
   is the run that produces them.
6. Dependency audits (cargo audit and npm audit): wired into the CI
   workflow as report steps. They were not run in this environment
   because cargo audit is not installed here; CI will show their real
   output on the first run.
