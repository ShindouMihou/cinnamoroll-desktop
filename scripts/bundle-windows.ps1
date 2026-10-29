$ErrorActionPreference = 'Stop'
$ProjectDir = Split-Path -Parent $PSScriptRoot
Push-Location $ProjectDir
try {
    $BuildFeatures = "bundled-sdl,updater"
    cargo build --release --locked --features $BuildFeatures
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
    $Destination = Join-Path $ProjectDir 'dist/Cinnamoroll-Windows'
    New-Item -ItemType Directory -Force -Path $Destination | Out-Null
    Copy-Item 'target/release/cinnamoroll-desktop.exe' (Join-Path $Destination 'Cinnamoroll.exe')
    $Library = Get-ChildItem 'target/release/build' -Recurse -Filter SDL3.dll |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $Library) { throw 'SDL3.dll was not found in the bundled build' }
    Copy-Item $Library.FullName $Destination
    Copy-Item README.md, LICENSE, THIRD_PARTY_NOTICES.md $Destination
    Copy-Item -Recurse -Force docs $Destination
    Copy-Item packaging/Windows-Readme.txt (Join-Path $Destination 'Read-me-first.txt')
    Copy-Item scripts/test-windows-package.ps1 (Join-Path $Destination 'Check-Windows.ps1')
    if (Test-Path 'packaging/licenses') { Copy-Item -Recurse -Force 'packaging/licenses' $Destination }
    Compress-Archive -Force -Path $Destination -DestinationPath 'dist/Cinnamoroll-Windows.zip'
    Write-Output "Built $Destination"
} finally {
    Pop-Location
}
