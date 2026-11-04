# Phase 0: hardware spike report

Status: the spike program is built and its logic is tested. The three
hardware questions are **not yet answered**, because this build was made on
a Linux machine with no Windows laptop attached. Anyone who claims these
answers without the run described below would be guessing. The spec is
explicit: do not continue past Phase 0 on assumptions, so the answers are
recorded here as open, with the exact run that closes them.

## The spike

Crate: `crates/alive_spike`. One console program that reads the current
power values, saves a snapshot plus a dirty marker, applies lid action Do
nothing for AC and DC and never timeouts, holds the keep awake state,
waits, and then restores everything on Enter.

Verified in this build: on a non Windows machine the spike prints the
Phase 0 questions, reports that reading power values is unsupported here,
states that no power setting was changed, and exits with code 2. That
behaviour was run and observed in this build session.

## The run that answers the questions (Windows laptop required)

1. Build on the laptop: `cargo run -p alive_spike`
2. Before starting, run `powercfg /a` and save the output with the script
   in `scripts/get-power-state.ps1`. Record whether the machine shows
   Standby (S0 Low Power Idle) or Standby (S3).
3. Start the two demo agents from `tests/hardware`:
   `python demo_agent.py heartbeat.log` and
   `python demo_cloud_agent.py cloud.log`
4. Start the spike, close the lid, and leave it closed for 30 minutes,
   first plugged in and then, on a second run, on battery.
5. Open the lid, press Enter in the spike so it restores, and check:
   the heartbeat file for gaps over 15 seconds, the cloud log for failed
   calls, and the battery percent before and after.
6. Fill in the answers below and attach the power state captures.

## Questions

1. Does applying the lid action and the timeouts need elevation on a
   standard user account?
   Answer: OPEN. To be measured by running the spike from a standard,
   non administrator account and recording whether the power steps fail.
   If elevation is needed, the design in section 5.3 of the spec applies:
   a minimal single purpose elevated helper, chosen and documented here.
2. Does the demo agent keep running on battery with the lid closed on a
   Modern Standby machine?
   Answer: OPEN. To be measured by the heartbeat gap check in the run
   above on an S0 machine.
3. What is the battery drain?
   Answer: OPEN. To be measured as percent drop over the 30 minute closed
   lid run, compared with an OFF baseline with the lid open and the
   display off.

## Hypotheses carried from the spec, still unverified

1. Spec item 5 in section 3: on Modern Standby on battery, Windows may
   ignore or terminate power requests, so Alive also sets the idle
   timeouts to never. The spike sets those timeouts, so the run tests the
   combined behaviour, which is what the product relies on.
2. Spec item 8 in section 3: EcoQoS throttling for heavy agents. Not used
   by the spike; it belongs to the Phase 3 supervisor and is verified
   there on Windows.
