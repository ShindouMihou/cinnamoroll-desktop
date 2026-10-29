param([Parameter(Mandatory)][string]$ArchivePath)
$ErrorActionPreference = 'Stop'
$Archive = Get-Item -LiteralPath $ArchivePath
$NameMatch = [regex]::Match($Archive.Name, '^Cinnamoroll-([0-9]+\.[0-9]+\.[0-9]+)-x86_64-pc-windows\.zip$')
if (-not $NameMatch.Success) { throw 'The test needs a Windows release ZIP.' }
$Version = $NameMatch.Groups[1].Value
$Root = Join-Path ([IO.Path]::GetTempPath()) ('cinnamoroll-installer-check-' + [guid]::NewGuid().ToString('N'))
$Destination = Join-Path $Root 'Cinnamoroll'
$Checksums = Join-Path $Root 'SHA256SUMS'
$Installer = Join-Path (Split-Path $PSScriptRoot -Parent) 'install.ps1'
New-Item -ItemType Directory -Path $Root | Out-Null
try {
    $Hash = (Get-FileHash -LiteralPath $Archive.FullName -Algorithm SHA256).Hash
    Set-Content -LiteralPath $Checksums -Value "$Hash  $($Archive.Name)"
    & $Installer -Version $Version -ArchivePath $Archive.FullName -ChecksumPath $Checksums -InstallRoot $Destination -NoLaunch -NoShortcuts
    $Executable = Join-Path $Destination 'Cinnamoroll.exe'
    if (-not (Test-Path -LiteralPath $Executable)) { throw 'The installer did not create the app.' }
    $Sentinel = Join-Path $Destination 'preserve-on-failure.txt'
    Set-Content -LiteralPath $Sentinel -Value 'keep this on failure'
    Set-Content -LiteralPath $Checksums -Value "$('0' * 64)  $($Archive.Name)"
    $Rejected = $false
    try { & $Installer -Version $Version -ArchivePath $Archive.FullName -ChecksumPath $Checksums -InstallRoot $Destination -NoLaunch -NoShortcuts }
    catch { $Rejected = $true }
    if (-not $Rejected -or -not (Test-Path -LiteralPath $Sentinel)) { throw 'Checksum rejection did not preserve the previous installation.' }
    Set-Content -LiteralPath $Checksums -Value "$Hash  $($Archive.Name)"
    & $Installer -Version $Version -ArchivePath $Archive.FullName -ChecksumPath $Checksums -InstallRoot $Destination -NoLaunch -NoShortcuts
    if (Test-Path -LiteralPath $Sentinel) { throw 'Successful replacement left stale files behind.' }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $Unsafe = Join-Path $Root 'unsafe.zip'
    $Zip = [IO.Compression.ZipFile]::Open($Unsafe, [IO.Compression.ZipArchiveMode]::Create)
    $null = $Zip.CreateEntry('../outside.txt')
    $Zip.Dispose()
    $UnsafeHash = (Get-FileHash -LiteralPath $Unsafe -Algorithm SHA256).Hash
    Set-Content -LiteralPath $Checksums -Value "$UnsafeHash  $($Archive.Name)"
    $Rejected = $false
    try { & $Installer -Version $Version -ArchivePath $Unsafe -ChecksumPath $Checksums -InstallRoot $Destination -NoLaunch -NoShortcuts }
    catch { $Rejected = $true }
    if (-not $Rejected -or (Test-Path -LiteralPath (Join-Path $Root 'outside.txt'))) { throw 'Archive traversal was not rejected.' }
    Write-Output 'Windows installer: install, replace, checksum rejection, and traversal checks passed.'
} finally {
    Remove-Item -LiteralPath $Root -Recurse -Force
}
