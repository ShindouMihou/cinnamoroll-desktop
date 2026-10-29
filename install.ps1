[CmdletBinding()]
param(
    [string]$Version,
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA 'Programs\Cinnamoroll'),
    [string]$ArchivePath,
    [string]$ChecksumPath,
    [switch]$NoLaunch,
    [switch]$NoShortcuts
)
$ErrorActionPreference = 'Stop'
$Repository = 'ShindouMihou/cinnamoroll-desktop'
if ($env:OS -ne 'Windows_NT') { throw 'On macOS, use install.sh.' }
if (-not [Environment]::Is64BitOperatingSystem) { throw 'Cinnamoroll needs 64-bit Windows.' }
if ($PSVersionTable.PSVersion.Major -lt 6) {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
}
$InstallRoot = [IO.Path]::GetFullPath($InstallRoot).TrimEnd('\')
if ((Split-Path $InstallRoot -Leaf) -ne 'Cinnamoroll') {
    throw 'The install directory must end in Cinnamoroll so unrelated folders cannot be replaced.'
}
$Temporary = Join-Path ([IO.Path]::GetTempPath()) ('cinnamoroll-' + [guid]::NewGuid().ToString('N'))
$Stage = $null
$Backup = $null
New-Item -ItemType Directory -Path $Temporary | Out-Null

function Get-ReleaseFile([string]$Url, [string]$Path) {
    if (-not $Url.StartsWith('https://')) { throw 'Downloads must use HTTPS.' }
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Path -TimeoutSec 180
    if ((Get-Item -LiteralPath $Path).Length -gt 134217728) { throw 'The download exceeds the release size limit.' }
}

function Test-ArchivePaths([string]$Path) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $Zip = [IO.Compression.ZipFile]::OpenRead($Path)
    try {
        [long]$Expanded = 0
        if ($Zip.Entries.Count -gt 5000) { throw 'The archive has too many files.' }
        foreach ($Entry in $Zip.Entries) {
            $Name = $Entry.FullName.Replace('\', '/')
            if ($Name.StartsWith('/') -or $Name -match '(^|/)\.\.(/|$)|:' -or (($Entry.ExternalAttributes -shr 16) -band 61440) -eq 40960) {
                throw 'The archive contains an unsafe path or symbolic link.'
            }
            $Expanded += $Entry.Length
            if ($Expanded -gt 536870912) { throw 'The extracted app exceeds its size limit.' }
        }
    } finally { $Zip.Dispose() }
}

try {
    if (-not $Version) {
        if ($ArchivePath) { throw 'Offline installation needs -Version.' }
        try {
            $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases/latest" -Headers @{ 'User-Agent' = 'Cinnamoroll-installer' } -TimeoutSec 30
            $Version = $Release.tag_name
        } catch { throw 'No published release is available yet. Check the repository Releases page.' }
    }
    $Version = $Version -replace '^v', ''
    if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw 'Use a stable version such as 0.1.1.' }
    $Asset = "Cinnamoroll-$Version-x86_64-pc-windows.zip"
    $Base = "https://github.com/$Repository/releases/download/v$Version"
    if (-not $ArchivePath) {
        $ArchivePath = Join-Path $Temporary $Asset
        Write-Host "Fetching Cinnamoroll $Version..."
        Get-ReleaseFile "$Base/$Asset" $ArchivePath
    }
    if (-not $ChecksumPath) {
        $ChecksumPath = Join-Path $Temporary 'SHA256SUMS'
        Get-ReleaseFile "$Base/SHA256SUMS" $ChecksumPath
    }
    $MatchesForAsset = @(Get-Content -LiteralPath $ChecksumPath | Where-Object { $_ -match ('^[a-fA-F0-9]{64}\s+\*?' + [regex]::Escape($Asset) + '$') })
    if ($MatchesForAsset.Count -ne 1) { throw 'The release must contain exactly one checksum for this app.' }
    $Expected = ($MatchesForAsset[0] -split '\s+')[0]
    if ((Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash -ne $Expected) { throw 'Checksum mismatch. Nothing was installed.' }
    Test-ArchivePaths $ArchivePath
    $Parent = Split-Path $InstallRoot -Parent
    New-Item -ItemType Directory -Force -Path $Parent | Out-Null
    $ExistingExecutable = Join-Path $InstallRoot 'Cinnamoroll.exe'
    foreach ($Process in @(Get-Process -Name Cinnamoroll -ErrorAction SilentlyContinue)) {
        if ($Process.Path -eq $ExistingExecutable) { throw 'Quit the installed Cinnamoroll before running this installer again.' }
    }
    $Stage = Join-Path $Parent ('.cinnamoroll-install-' + [guid]::NewGuid().ToString('N'))
    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $Stage
    $Payload = Join-Path $Stage 'Cinnamoroll-Windows'
    $Executable = Join-Path $Payload 'Cinnamoroll.exe'
    if (-not (Test-Path -LiteralPath $Executable) -or -not (Test-Path -LiteralPath (Join-Path $Payload 'SDL3.dll'))) {
        throw 'The archive is missing the executable or SDL3.dll.'
    }
    $Output = Join-Path $Temporary 'version.txt'
    $Probe = Start-Process -FilePath $Executable -ArgumentList '--version' -RedirectStandardOutput $Output -PassThru -WindowStyle Hidden
    if (-not $Probe.WaitForExit(15000)) { $Probe.Kill(); throw 'The app did not pass its launch check.' }
    if ($Probe.ExitCode -ne 0 -or (Get-Content -LiteralPath $Output -Raw).Trim() -ne "Cinnamoroll $Version") {
        throw 'The app does not match the release version.'
    }
    if (Test-Path -LiteralPath $InstallRoot) {
        if (-not (Test-Path -LiteralPath $ExistingExecutable)) { throw 'The destination contains an unrecognized app; it was left untouched.' }
        $Backup = Join-Path $Parent ('.cinnamoroll-previous-' + [guid]::NewGuid().ToString('N'))
        Move-Item -LiteralPath $InstallRoot -Destination $Backup
    }
    Move-Item -LiteralPath $Payload -Destination $InstallRoot
    if ($Backup) { Remove-Item -LiteralPath $Backup -Recurse -Force; $Backup = $null }
    if (-not $NoShortcuts) {
        $Programs = [Environment]::GetFolderPath('Programs')
        if (-not $Programs) { $Programs = Join-Path $env:APPDATA 'Microsoft/Windows/Start Menu/Programs' }
        New-Item -ItemType Directory -Force -Path $Programs | Out-Null
        $Shell = New-Object -ComObject WScript.Shell
        $Shortcut = $Shell.CreateShortcut((Join-Path $Programs 'Cinnamoroll.lnk'))
        $Shortcut.TargetPath = Join-Path $InstallRoot 'Cinnamoroll.exe'
        $Shortcut.WorkingDirectory = $InstallRoot
        $Shortcut.Description = 'A little desktop company'
        $Shortcut.Save()
    }
    Write-Host "Cinnamoroll $Version is ready at $InstallRoot"
    if (-not $NoLaunch) {
        if (Get-Process -Name Cinnamoroll -ErrorAction SilentlyContinue) {
            Write-Host 'Quit the running copy, then open Cinnamoroll from Start when ready.'
        } else { Start-Process -FilePath (Join-Path $InstallRoot 'Cinnamoroll.exe') }
    }
} catch {
    if ($Backup -and (Test-Path -LiteralPath $Backup) -and -not (Test-Path -LiteralPath $InstallRoot)) {
        try { Move-Item -LiteralPath $Backup -Destination $InstallRoot; $Backup = $null }
        catch { Write-Warning "Your previous app is preserved at $Backup" }
    }
    throw
} finally {
    if ($Stage -and (Test-Path -LiteralPath $Stage)) { Remove-Item -LiteralPath $Stage -Recurse -Force }
    Remove-Item -LiteralPath $Temporary -Recurse -Force
}
