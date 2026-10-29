$ErrorActionPreference = 'Stop'
$Executable = Join-Path $PSScriptRoot 'Cinnamoroll.exe'
if (-not (Test-Path $Executable)) { throw 'Keep this script beside Cinnamoroll.exe in the extracted folder.' }
if (-not (Test-Path (Join-Path $PSScriptRoot 'SDL3.dll'))) { throw 'SDL3.dll is missing. Extract the complete ZIP first.' }
$Process = Start-Process -FilePath $Executable -ArgumentList '--smoke-seconds', '5' -PassThru
if (-not $Process.WaitForExit(20000)) {
    $Process.Kill()
    throw 'The launch check timed out.'
}
if ($Process.ExitCode -ne 0) { throw "Cinnamoroll exited with code $($Process.ExitCode). Quit any existing pet before running this check." }
Write-Output 'The packaged app launched and exited successfully. Now open Cinnamoroll.exe normally and check dragging, settings, and the tray icon.'
