# Manual hardware test sheets for Alive

These tests need a real Windows laptop. They cannot be run in a Linux
build environment, and their results must be written into
`docs/test-report.md` only after they actually run. The demo agents in
this folder are the ones the spec names: `demo_agent.py` writes a
heartbeat timestamp every 10 seconds, and `demo_cloud_agent.py` calls a
public HTTPS endpoint every minute and logs the result.

For every test, first capture the starting state with
`scripts/get-power-state.ps1`, and note the laptop model, the Windows
build, whether `powercfg /a` shows S0 Modern Standby or S3, and the
battery percent at the start and at the end.

1. TC31: ON, lid closed on mains power for 30 minutes. Pass when the
   heartbeat has no gap over 15 seconds and the HTTPS calls succeed.
2. TC32: ON, lid closed on battery for 30 minutes. Same pass rule as
   TC31. Record the battery drop and compare it with an OFF baseline
   with the lid open and the display off.
3. TC33: OFF, lid closed. Pass when the machine sleeps, the agents are
   suspended, and after opening the lid Alive reports OFF with nothing
   changed.
4. TC34: ON, then open the lid. Pass when the display returns normally
   and nothing needs to be restored.
5. TC35: Repeat TC32 on a Modern Standby machine and on an S3 machine
   if one is available. Record any difference in full.
6. TC36: WiFi with the lid closed. Pass when the cloud demo agent keeps
   reaching its endpoint for the whole run.
7. TC37: Run `powercfg /sleepstudy` and `powercfg /batteryreport`
   before and after a long run and attach the findings to the report.
8. TC38: Efficiency budgets. Measure idle CPU over 10 minutes with no
   active agents (target under 0.5 percent), resident memory with the
   window closed (target under 30 MB for the Rust backend), and the
   window open time (target under 600 ms). Record the real numbers.
9. TC39: Port check. Pass when a listening port listing shows that
   Alive opens no network ports.
10. TC40: Installer. Install per user with no administrator rights,
    launch, toggle ON and OFF, uninstall and confirm the power settings
    are restored, then reinstall as an upgrade and confirm the config
    is kept.
11. TC41: Scaling at 100, 150 and 200 percent, and on more than one
    monitor. Pass when every page stays usable with no clipped controls.
12. TC42: Tray. Pass when the icon changes for every state, the tooltip
    text is correct, every menu item works, and Quit restores
    everything.
