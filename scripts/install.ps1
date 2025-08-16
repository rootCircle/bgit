param(
  [ValidateSet('install','update','uninstall','purge')]
  [string]$Cmd = 'install',
  [string]$Tag = "",
  [string]$To = ""
)

$ErrorActionPreference = 'Stop'

# Usage: iwr -useb https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.ps1 | iex

$Repo = 'rootCircle/bgit'
$Bin = 'bgit.exe'

function Get-DefaultInstallDir {
  $localBin = Join-Path $env:LOCALAPPDATA 'Programs\bgit'
  return $localBin
}

function Invoke-Http($Url) {
  (Invoke-WebRequest -UseBasicParsing -Uri $Url).Content
}

function Get-LatestTag {
  $json = Invoke-Http "https://api.github.com/repos/$Repo/releases/latest" | ConvertFrom-Json
  return $json.tag_name
}

function Resolve-Tag {
  if (-not $script:Tag) { $script:Tag = Get-LatestTag }
  if (-not $script:Tag) { throw 'Could not determine latest release tag' }
}

# Map OS label and arch to artifact naming
$OsLabel = 'windows-latest'
$Arch = $env:PROCESSOR_ARCHITECTURE
# Normalize arch
switch -regex ($Arch) {
  'AMD64|X86_64' { $Arch = 'AMD64' }
  'ARM64' { $Arch = 'ARM64' }
}

function Install-Bgit {
  Resolve-Tag
  $AssetZip = "bgit-$Tag-$OsLabel-$Arch.zip"
  $AssetSha = "$AssetZip.sha256"

  $rel = Invoke-Http "https://api.github.com/repos/$Repo/releases/tags/$Tag" | ConvertFrom-Json
  $urlZip = ($rel.assets | Where-Object { $_.name -eq $AssetZip }).browser_download_url
  $urlSha = ($rel.assets | Where-Object { $_.name -eq $AssetSha }).browser_download_url
  if (-not $urlZip) { throw "Could not find asset $AssetZip" }

  $tmp = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid())
  try {
    $zipPath = Join-Path $tmp.FullName $AssetZip
    $shaPath = Join-Path $tmp.FullName $AssetSha
    Invoke-WebRequest -UseBasicParsing -Uri $urlZip -OutFile $zipPath
    if ($urlSha) { Invoke-WebRequest -UseBasicParsing -Uri $urlSha -OutFile $shaPath }

    if (Test-Path $shaPath) {
      $expected = (Get-Content $shaPath).Split(' ')[0].Trim().ToLower()
      $actual = (Get-FileHash $zipPath -Algorithm SHA256).Hash.ToLower()
      if ($expected -ne $actual) { throw "Checksum mismatch for $AssetZip" }
    }

    $extractDir = Join-Path $tmp.FullName 'extract'
    New-Item -ItemType Directory -Force -Path $extractDir | Out-Null
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::ExtractToDirectory($zipPath, $extractDir)

    $src = Join-Path $extractDir $Bin
    if (-not (Test-Path $src)) { throw 'Binary not found in archive' }

    if (-not $To) { $To = Get-DefaultInstallDir }
    New-Item -ItemType Directory -Force -Path $To | Out-Null
    $dst = Join-Path $To $Bin
    Copy-Item -Force $src $dst

    $onPath = ($env:PATH -split ';') -contains $To
    if (-not $onPath) {
      Write-Host "Note: $To is not on PATH. Add it via System Properties or:`n  setx PATH `"$env:PATH;$To`""
    }

    Write-Host "Installed bgit $Tag to $dst"
  }
  finally {
    Remove-Item -Recurse -Force $tmp
  }
}

function Uninstall-Bgit {
  $candidate = $null
  if ($To) {
    $p = Join-Path $To $Bin
    if (Test-Path $p) { $candidate = $p }
  }
  if (-not $candidate) {
    $pathHit = (Get-Command bgit.exe -ErrorAction SilentlyContinue)
    if ($pathHit) { $candidate = $pathHit.Source }
  }
  if (-not $candidate) {
    $common = @(
      Join-Path $env:LOCALAPPDATA 'Programs\bgit\bgit.exe'),
      'C:\\Program Files\\bgit\\bgit.exe'
    foreach ($c in $common) { if (Test-Path $c) { $candidate = $c; break } }
  }
  if (-not $candidate) { Write-Host 'bgit not found; nothing to uninstall'; return }
  Remove-Item -Force $candidate
  Write-Host "Removed $candidate"
}

function Purge-Bgit {
  Uninstall-Bgit
  $agentSock = Join-Path $env:USERPROFILE '.ssh\bgit_ssh_agent.sock'
  if (Test-Path $agentSock) { Remove-Item -Force $agentSock; Write-Host "Removed $agentSock" }
  # Global per-user config in %APPDATA%\bgit
  $globalCfg = Join-Path $env:APPDATA 'bgit'
  if (Test-Path $globalCfg) { Remove-Item -Recurse -Force $globalCfg; Write-Host "Removed $globalCfg" }
}

switch ($Cmd) {
  'install' { Install-Bgit }
  'update' { Install-Bgit }
  'uninstall' { Uninstall-Bgit }
  'purge' { Purge-Bgit }
}
