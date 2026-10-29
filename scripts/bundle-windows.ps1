$ErrorActionPreference = 'Stop'
$ProjectDir = Split-Path -Parent $PSScriptRoot
Push-Location $ProjectDir
$PreviousToolchain = $env:CMAKE_TOOLCHAIN_FILE
try {
    $Toolchain = Join-Path $ProjectDir 'target/cinnamoroll-msvc-runtime.cmake'
    New-Item -ItemType Directory -Force -Path (Split-Path $Toolchain -Parent) | Out-Null
    $Settings = @()
    if ($PreviousToolchain) {
        $Existing = [IO.Path]::GetFullPath($PreviousToolchain).Replace('\', '/')
        $Settings += 'include("' + $Existing + '")'
    }
    $Settings += 'set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded" CACHE STRING "Cinnamoroll portable runtime" FORCE)'
    [IO.File]::WriteAllLines($Toolchain, $Settings, [Text.UTF8Encoding]::new($false))
    $env:CMAKE_TOOLCHAIN_FILE = $Toolchain
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
    $VsWhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $Dumpbin = & $VsWhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find 'VC\Tools\MSVC\**\bin\Hostx64\x64\dumpbin.exe' | Select-Object -First 1
    if (-not $Dumpbin) { throw 'Cannot inspect native runtime dependencies; Visual Studio C++ Build Tools are required.' }
    foreach ($Binary in @((Join-Path $Destination 'Cinnamoroll.exe'), (Join-Path $Destination 'SDL3.dll'))) {
        $Imports = & $Dumpbin /dependents $Binary
        if ($LASTEXITCODE -ne 0) { throw 'Could not inspect the packaged executable or SDL library.' }
        if ($Imports -match '(?i)\b(vcruntime|msvcp|msvcr)[0-9][a-z0-9_]*\.dll\b') {
            throw "Portable runtime check failed for $Binary : $($Imports -join [Environment]::NewLine)"
        }
    }
    Copy-Item README.md, LICENSE, THIRD_PARTY_NOTICES.md $Destination
    Copy-Item -Recurse -Force docs $Destination
    Copy-Item packaging/Windows-Readme.txt (Join-Path $Destination 'Read-me-first.txt')
    Copy-Item scripts/test-windows-package.ps1 (Join-Path $Destination 'Check-Windows.ps1')
    if (Test-Path 'packaging/licenses') { Copy-Item -Recurse -Force 'packaging/licenses' $Destination }
    Compress-Archive -Force -Path $Destination -DestinationPath 'dist/Cinnamoroll-Windows.zip'
    Write-Output "Built $Destination"
} finally {
    $env:CMAKE_TOOLCHAIN_FILE = $PreviousToolchain
    Pop-Location
}
