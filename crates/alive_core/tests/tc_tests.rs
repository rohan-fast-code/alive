use alive_core::mocks::MockSystem;
use alive_core::snapshot;
use alive_core::traits::*;
use alive_core::*;
use std::path::PathBuf;

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

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("alive_tc_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn tc01_toggle_on_emits_exact_ordered_sequence() {
    let mut sys = MockSystem::new(original());
    let mut orch = Orchestrator::new();
    orch.toggle_on(&mut sys).unwrap();
    assert_eq!(
        sys.log,
        vec![
            "snapshot",
            "guard",
            "lid",
            "timeouts",
            "hold",
            "subscribe",
            "agents"
        ]
    );
    assert!(orch.state.is_on());
    assert!(sys.is_held());
    assert!(GuardControl::guard_running(&sys));
    // Lid and timeouts were actually applied.
    assert_eq!(sys.values.lid_ac, 0);
    assert_eq!(sys.values.lid_dc, 0);
    assert_eq!(sys.values.standby_ac_secs, 0);
}

#[test]
fn tc02_toggle_off_restores_every_setting_exactly() {
    let orig = original();
    let mut sys = MockSystem::new(orig);
    let mut orch = Orchestrator::new();
    orch.toggle_on(&mut sys).unwrap();
    orch.toggle_off(&mut sys).unwrap();
    assert_eq!(
        sys.values, orig,
        "AC and DC values must be restored exactly"
    );
    assert_eq!(orch.state, State::Off);
    assert!(!sys.is_held());
    assert!(!GuardControl::guard_running(&sys));
    // Restoring is idempotent: a second OFF changes nothing.
    orch.toggle_off(&mut sys).unwrap();
    assert_eq!(sys.values, orig);
    assert_eq!(orch.state, State::Off);
}

#[test]
fn tc03_fifty_toggles_end_consistent_with_no_leaked_hold() {
    let orig = original();
    let mut sys = MockSystem::new(orig);
    let mut orch = Orchestrator::new();
    for _ in 0..25 {
        orch.toggle_on(&mut sys).unwrap();
        orch.toggle_off(&mut sys).unwrap();
    }
    assert_eq!(orch.state, State::Off);
    assert!(
        !sys.is_held(),
        "no hold may leak after an even number of toggles"
    );
    assert_eq!(sys.values, orig);
    assert!(!GuardControl::guard_running(&sys));
}

#[test]
fn tc04_failure_at_each_arming_step_undoes_in_reverse_and_ends_off() {
    let steps = [
        "snapshot",
        "guard",
        "lid",
        "timeouts",
        "hold",
        "subscribe",
        "agents",
    ];
    for step in steps {
        let orig = original();
        let mut sys = MockSystem::failing(orig, step);
        let mut orch = Orchestrator::new();
        let result = orch.toggle_on(&mut sys);
        assert!(result.is_err(), "step {step} should fail");
        assert_eq!(orch.state, State::Off, "step {step} must end in OFF");
        assert!(!sys.is_held(), "step {step} leaked a hold");
        assert!(
            !GuardControl::guard_running(&sys),
            "step {step} leaked the guard"
        );
        assert_eq!(sys.values, orig, "step {step} left changed power values");
        // Undo ran in reverse: the log tail after the failed step must not
        // contain any step that comes after the failed one as a new action.
        let failed_pos = sys.log.iter().position(|l| l == step).unwrap();
        let tail = &sys.log[failed_pos + 1..];
        for later in steps.iter().skip_while(|s| **s != step).skip(1) {
            assert!(
                !tail.iter().any(|l| l == later),
                "step {step}: later step {later} ran after the failure"
            );
        }
    }
}

#[test]
fn tc05_policy_on_ac_allows_all_at_any_battery_level() {
    let mut engine = PolicyEngine::default();
    for percent in [100u8, 50, 28, 14, 1] {
        assert_eq!(
            engine.evaluate(Mode::Smart, PowerSource::Ac, Some(percent), false),
            PolicyDecision::AllowAll,
            "AC at {percent} percent"
        );
        assert_eq!(
            engine.evaluate(Mode::Strict, PowerSource::Ac, Some(percent), false),
            PolicyDecision::AllowAll
        );
    }
}

#[test]
fn tc06_smart_on_battery_at_80_allows_all() {
    let mut engine = PolicyEngine::default();
    assert_eq!(
        engine.evaluate(Mode::Smart, PowerSource::Battery, Some(80), false),
        PolicyDecision::AllowAll
    );
}

#[test]
fn tc07_smart_on_battery_at_28_pauses_heavy_only() {
    let mut engine = PolicyEngine::default();
    assert_eq!(
        engine.evaluate(Mode::Smart, PowerSource::Battery, Some(28), false),
        PolicyDecision::PauseHeavy
    );
}

#[test]
fn tc08_battery_at_14_turns_alive_off() {
    let mut smart = PolicyEngine::default();
    assert_eq!(
        smart.evaluate(Mode::Smart, PowerSource::Battery, Some(14), false),
        PolicyDecision::CriticalOff
    );
    let mut strict = PolicyEngine::default();
    assert_eq!(
        strict.evaluate(Mode::Strict, PowerSource::Battery, Some(14), false),
        PolicyDecision::CriticalOff
    );
}

#[test]
fn tc09_hysteresis_prevents_flapping_between_29_and_31() {
    let mut engine = PolicyEngine::default();
    let mut decisions = Vec::new();
    for i in 0..20 {
        let percent = if i % 2 == 0 { 29 } else { 31 };
        decisions.push(engine.evaluate(Mode::Smart, PowerSource::Battery, Some(percent), false));
    }
    let transitions = decisions.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(
        transitions <= 1,
        "expected at most one transition, got {transitions}: {decisions:?}"
    );
    assert_eq!(decisions[0], PolicyDecision::PauseHeavy);
}

#[test]
fn tc10_battery_saver_pauses_heavy_regardless_of_percent() {
    let mut engine = PolicyEngine::default();
    assert_eq!(
        engine.evaluate(Mode::Smart, PowerSource::Battery, Some(95), true),
        PolicyDecision::PauseHeavy
    );
}

#[test]
fn tc11_plugged_in_only_mode_on_battery_waits_for_power() {
    let mut engine = PolicyEngine::default();
    assert_eq!(
        engine.evaluate(Mode::PluggedInOnly, PowerSource::Battery, Some(90), false),
        PolicyDecision::WaitingForPower
    );
    assert_eq!(
        engine.evaluate(Mode::PluggedInOnly, PowerSource::Ac, Some(10), false),
        PolicyDecision::AllowAll
    );
}

fn sample_agent(name: &str, command: &str) -> AgentConfig {
    AgentConfig {
        id: name.to_string(),
        name: name.to_string(),
        command: command.to_string(),
        args: vec![],
        working_dir: String::new(),
        env: Default::default(),
        kind: AgentKind::Cloud,
        heavy: false,
        enabled: true,
        autostart: true,
        restart_policy: RestartPolicy::OnFailure,
        max_restarts: 5,
        backoff_start_seconds: 2,
        backoff_max_seconds: 60,
        stable_after_seconds: 120,
        health: HealthKind::None,
        health_target: String::new(),
        health_interval_seconds: 0,
    }
}

#[test]
fn tc12_config_validation_rejects_bad_agents() {
    // Missing command
    let mut cfg = AliveConfig::default();
    cfg.agents.push(sample_agent("a", ""));
    assert!(cfg
        .validate(false)
        .iter()
        .any(|e| e.contains("command is missing")));

    // Duplicate names
    let mut cfg = AliveConfig::default();
    cfg.agents.push(sample_agent("dup", "/bin/true"));
    cfg.agents.push(sample_agent("dup", "/bin/true"));
    assert!(cfg.validate(false).iter().any(|e| e.contains("duplicate")));

    // Nonexistent path, checked explicitly
    let mut cfg = AliveConfig::default();
    cfg.agents.push(sample_agent(
        "ghost",
        "/definitely/not/a/real/program_alive_test",
    ));
    assert!(cfg
        .validate(true)
        .iter()
        .any(|e| e.contains("does not exist")));

    // Negative numbers and oversized values
    let mut bad = sample_agent("neg", "/bin/true");
    bad.max_restarts = -1;
    assert!(!bad.validate(false).is_empty());
    let mut big = sample_agent("big", "/bin/true");
    big.max_restarts = 10_000;
    assert!(!big.validate(false).is_empty());

    // A good config passes when path checks use a real file.
    let dir = temp_dir("tc12");
    let exe = dir.join("fake_agent");
    std::fs::write(&exe, b"#!/bin/sh\n").unwrap();
    let mut cfg = AliveConfig::default();
    cfg.agents.push(sample_agent("good", exe.to_str().unwrap()));
    assert!(
        cfg.validate(true).is_empty(),
        "errors: {:?}",
        cfg.validate(true)
    );
}

#[test]
fn tc13_snapshot_roundtrip_and_corruption_fallback() {
    let dir = temp_dir("tc13");
    let snap = Snapshot::new(original(), true);
    snapshot::save_snapshot(&dir, &snap).unwrap();
    assert!(snapshot::is_dirty(&dir));
    let loaded = snapshot::load_snapshot(&dir).unwrap();
    assert_eq!(loaded, snap);
    snapshot::clear_dirty(&dir).unwrap();
    assert!(!snapshot::is_dirty(&dir));

    // Truncated file is detected as corrupt.
    std::fs::write(dir.join("snapshot.json"), b"{\"version\":1,\"val").unwrap();
    assert!(matches!(
        snapshot::load_snapshot(&dir),
        Err(CoreError::CorruptSnapshot)
    ));
    // Garbage file is detected as corrupt.
    std::fs::write(dir.join("snapshot.json"), b"not json at all").unwrap();
    assert!(matches!(
        snapshot::load_snapshot(&dir),
        Err(CoreError::CorruptSnapshot)
    ));
    // The fallback that callers use in that case is the documented default set.
    let fallback = PowerValues::windows_fallback_defaults();
    assert_eq!(fallback.lid_ac, 1);
    assert_eq!(fallback.lid_dc, 1);
}

#[test]
fn tc14_backoff_is_exponential_capped_and_resets_after_stable() {
    let backoff = Backoff::new(2.0, 30.0);
    let d0 = backoff.delay_seconds(0);
    let d1 = backoff.delay_seconds(1);
    let d2 = backoff.delay_seconds(2);
    assert!(d1 > d0, "backoff must grow: {d0} then {d1}");
    assert!(d2 > d1, "backoff must grow: {d1} then {d2}");
    for attempt in 0..20 {
        assert!(
            backoff.delay_seconds(attempt) <= 30.0,
            "backoff must be capped"
        );
        // Jitter stays within the documented band of the base delay.
        let base = backoff.base_delay_seconds(attempt);
        let d = backoff.delay_seconds(attempt);
        if base < 30.0 {
            assert!(
                d >= base * 0.80 && d <= base * 1.20,
                "jitter band: base {base} got {d}"
            );
        }
    }
    let mut tracker = RestartTracker::new(backoff, 120);
    tracker.record_failure();
    tracker.record_failure();
    assert_eq!(tracker.failures(), 2);
    tracker.record_run_duration(60);
    assert_eq!(
        tracker.failures(),
        2,
        "a short run must not reset the counter"
    );
    tracker.record_run_duration(120);
    assert_eq!(tracker.failures(), 0, "a stable run must reset the counter");
}

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// TC15: for any sequence of toggles, the invariants hold.
    /// If the state is OFF then no hold is held, the guard is not running,
    /// and every power value equals the original snapshot values.
    #[test]
    fn tc15_random_toggle_sequences_preserve_invariants(toggles in proptest::collection::vec(any::<bool>(), 0..60)) {
        let orig = original();
        let mut sys = MockSystem::new(orig);
        let mut orch = Orchestrator::new();
        for turn_on in toggles {
            if turn_on && orch.state == State::Off {
                orch.toggle_on(&mut sys).unwrap();
            } else if !turn_on && orch.state.is_on() {
                orch.toggle_off(&mut sys).unwrap();
            }
            if orch.state == State::Off {
                prop_assert!(!sys.is_held());
                prop_assert!(!GuardControl::guard_running(&sys));
                prop_assert_eq!(sys.values, orig);
            }
            // Restore idempotency at every OFF point.
            if orch.state == State::Off {
                orch.toggle_off(&mut sys).unwrap();
                prop_assert_eq!(sys.values, orig);
            }
        }
    }

    /// TC15 companion: any battery percent sequence in Smart mode never
    /// produces CriticalOff on AC, and never flaps more than the hysteresis
    /// band allows around the pause threshold.
    #[test]
    fn tc15_policy_never_critical_on_ac(percents in proptest::collection::vec(0u8..=100, 0..40)) {
        let mut engine = PolicyEngine::default();
        for p in percents {
            let d = engine.evaluate(Mode::Smart, PowerSource::Ac, Some(p), false);
            prop_assert_eq!(d, PolicyDecision::AllowAll);
        }
    }
}
