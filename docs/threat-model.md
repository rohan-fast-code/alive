# Alive threat model

Alive changes power settings and runs user defined programs. That is
powerful by design, so the risks and the mitigations are written down.

## Risks and mitigations

1. Changed power settings are left behind after a crash.
   Mitigation: the snapshot and dirty marker are written before any
   change; the guard sidecar restores after an unclean exit; startup
   restores a leftover dirty state before showing UI; restoring is
   idempotent; the uninstaller runs the same restore.
2. An agent definition is code execution.
   Mitigation: the Agents page shows a clear warning and the full
   command; nothing is imported silently; an import must be confirmed
   with the full command visible; commands are run as explicit argument
   lists with no shell, so an argument cannot become a new command.
3. Secrets leak through logs or diagnostics.
   Mitigation: environment values whose keys look like secrets are
   masked in logs, the UI and diagnostics exports; config exports remove
   secrets; no telemetry and no network calls are made by Alive itself.
4. A second copy of Alive fights over the settings.
   Mitigation: single instance is enforced; the second launch focuses
   the first window.
5. Alive hides what it is doing.
   Mitigation: the tray icon is always visible while ON, with a state
   specific icon and tooltip; Alive never disables or tampers with any
   Windows security feature.
6. Elevation is broader than needed.
   Mitigation: Alive runs as a standard user unless the Phase 0 result
   proves a power write needs elevation; in that case only a minimal
   single purpose helper is elevated, and the choice is documented in
   docs/phase0.md.
7. Battery or heat damage from a laptop running closed in a bag.
   Mitigation: the critical battery cutoff restores normal behaviour at
   the threshold; the docs and the first run welcome state plainly that
   a running laptop must never be carried closed in a bag.
8. A noisy or runaway agent exhausts memory or CPU.
   Mitigation (Phase 3 design): bounded log channels and ring buffers,
   optional per agent CPU and memory caps through the Job Object, and
   exponential backoff with a restart cap, after which the agent is
   marked Failed and the user is notified once.
