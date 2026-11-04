# End to end UI tests

WebDriver tests through tauri-driver with WebdriverIO cover TC26 to
TC30 from the design spec: the dashboard toggle and the backend state
staying in sync in both directions, inline errors when an agent is added
with invalid input, the Power Policy preview showing the true original
and applied values, full keyboard operation with a logical focus order,
and automated contrast checks in the light and dark themes.

These tests run on Windows, where tauri-driver and the Edge WebDriver
exist. They are part of the Phase 4 and Phase 6 work on a Windows
machine and are listed as not yet run in `docs/test-report.md`, instead
of being described as passing without evidence.
