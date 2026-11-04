# get-power-state.ps1
# Dumps the power facts that Alive tests need into one text file, so a
# hardware test run can be checked and attached to the test report.
# Run in PowerShell. No setting is changed by this script.

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$out = Join-Path $PSScriptRoot "power-state-$stamp.txt"

"Alive power state capture, $stamp" | Out-File $out
"" | Out-File $out -Append
"=== powercfg /a (sleep states, S0 Modern Standby or S3) ===" | Out-File $out -Append
powercfg /a | Out-File $out -Append
"" | Out-File $out -Append
"=== powercfg /requests (who is holding the system awake) ===" | Out-File $out -Append
powercfg /requests | Out-File $out -Append
"" | Out-File $out -Append
"=== Lid action, current values ===" | Out-File $out -Append
powercfg /query SCHEME_CURRENT SUB_BUTTONS LIDACTION | Out-File $out -Append
"" | Out-File $out -Append
"=== Sleep and hibernate timeouts, current values ===" | Out-File $out -Append
powercfg /query SCHEME_CURRENT SUB_SLEEP | Out-File $out -Append

Write-Host "Saved to $out"
