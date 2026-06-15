# SPDX-License-Identifier: MIT
#
# NailSnake: Cross-platform terminal Snake for Windows, Linux, and macOS
# Copyright (c) 2026 voltsparx
#
# Repository: https://github.com/voltsparx/NailSnake
# Contact: voltsparx@gmail.com
# License: See LICENSE file in the project root

[CmdletBinding()]
param(
  [Parameter(Position = 0)]
  [ValidateSet("install", "test", "uninstall", "update", "build", "check", "run")]
  [string]$Mode = "install",

  [Parameter(Position = 1, ValueFromRemainingArguments = $true)]
  [string[]]$Remaining
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$AppName = "nailsnake"
$ThemeTag = "NailSnake"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir
$InstallRoot = Join-Path $env:LOCALAPPDATA "Programs\$AppName"
$UserBinDir = Join-Path $HOME ".local\bin"
$SystemLauncher = Join-Path $UserBinDir "$AppName.cmd"
$TestBase = Join-Path $ScriptDir "test-root"
$TestRoot = Join-Path $TestBase $AppName
$TestBinDir = Join-Path $TestBase "bin"
$TestLauncher = Join-Path $TestBinDir "$AppName.cmd"
$ManifestName = ".nailsnake-install.json"
$ManSrc = Join-Path $RepoRoot "man" "$AppName.1"

function Write-Labelled {
  param(
    [string]$Label,
    [string]$Message,
    [ConsoleColor]$Color = [ConsoleColor]::Gray
  )
  Write-Host ("[{0}][{1}] {2}" -f $ThemeTag, $Label, $Message) -ForegroundColor $Color
}

function Write-Info {
  param([string]$Message)
  Write-Labelled -Label "INFO" -Message $Message -Color ([ConsoleColor]::Cyan)
}

function Write-Success {
  param([string]$Message)
  Write-Labelled -Label " OK " -Message $Message -Color ([ConsoleColor]::Green)
}

function Write-WarningLine {
  param([string]$Message)
  Write-Labelled -Label "WARN" -Message $Message -Color ([ConsoleColor]::Yellow)
}

function Stop-Step {
  param([string]$Message)
  Write-Labelled -Label "FAIL" -Message $Message -Color ([ConsoleColor]::Red)
  exit 1
}

function Read-Confirmation {
  param(
    [string]$Prompt,
    [bool]$Default = $true
  )

  if (-not [Environment]::UserInteractive) {
    return $true
  }

  $suffix = if ($Default) { "[Y/n]" } else { "[y/N]" }
  $answer = Read-Host "$Prompt $suffix"
  if ([string]::IsNullOrWhiteSpace($answer)) {
    return $Default
  }

  $normalized = $answer.Trim().ToLowerInvariant()
  return $normalized -in @("y", "yes")
}

function Confirm-CommandAvailable {
  param([string]$Name)

  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    Stop-Step "$Name is required but was not found in PATH."
  }
}

function Confirm-RustInstalled {
  Confirm-CommandAvailable -Name "cargo"
}

function Get-ManifestPath {
  param([string]$Root)
  Join-Path $Root $ManifestName
}

function Test-ManagedInstall {
  param([string]$Root)
  Test-Path -LiteralPath (Get-ManifestPath -Root $Root)
}

function Confirm-ManagedInstallTarget {
  param([string]$Root)

  if ((Test-Path -LiteralPath $Root) -and -not (Test-ManagedInstall -Root $Root)) {
    Stop-Step "Refusing to replace '$Root' because it is not marked as an NailSnake managed install."
  }
}

function Remove-TreeSafe {
  param([string]$PathToRemove)

  if ([string]::IsNullOrWhiteSpace($PathToRemove)) {
    return $true
  }

  if (-not (Test-Path -LiteralPath $PathToRemove)) {
    return $true
  }

  foreach ($attempt in 1..5) {
    try {
      Remove-Item -LiteralPath $PathToRemove -Recurse -Force -ErrorAction Stop
      return $true
    } catch {
      if ($attempt -lt 5) {
        Start-Sleep -Milliseconds 300
      }
    }
  }

  try {
    $parent = Split-Path -Parent $PathToRemove
    $leaf = Split-Path -Leaf $PathToRemove
    $quarantine = Join-Path $parent (".pending-delete-{0}-{1}" -f $leaf, [guid]::NewGuid().ToString("N"))
    Move-Item -LiteralPath $PathToRemove -Destination $quarantine -Force -ErrorAction Stop
    Write-WarningLine "Deferred cleanup for locked path: $quarantine"
    return $true
  } catch {
    Write-WarningLine "Unable to remove path immediately: $PathToRemove"
    return $false
  }
}

function Build-Release {
  Write-Info "Building release binary."
  & cargo build --release
  if ($LASTEXITCODE -ne 0) {
    Stop-Step "Release build failed."
  }
}

function Build-Debug {
  Write-Info "Building debug binary."
  & cargo build
  if ($LASTEXITCODE -ne 0) {
    Stop-Step "Debug build failed."
  }
}

function Invoke-Tests {
  Write-Info "Running tests."
  & cargo test
  if ($LASTEXITCODE -ne 0) {
    Stop-Step "Tests failed."
  }
}

function Copy-Binary {
  param([string]$DestDir)

  New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
  $binarySrc = Join-Path $RepoRoot "target\release\$AppName.exe"
  if (-not (Test-Path -LiteralPath $binarySrc)) {
    Stop-Step "Release binary not found at $binarySrc. Run 'build' mode first."
  }
  Copy-Item -LiteralPath $binarySrc -Destination (Join-Path $DestDir "$AppName.exe") -Force
}

function Install-ManPage {
  param([string]$AppRoot)

  if (-not (Test-Path -LiteralPath $ManSrc)) {
    Write-WarningLine "Man page not found at $ManSrc, skipping."
    return
  }

  $manDir = Join-Path $AppRoot "man\man1"
  New-Item -ItemType Directory -Path $manDir -Force | Out-Null
  Copy-Item -LiteralPath $ManSrc -Destination (Join-Path $manDir "nailsnake.1") -Force
  Write-Success "Man page installed."
}

function Write-Manifest {
  param(
    [string]$AppRoot,
    [string]$InstallMode
  )

  $cargoVersion = (& cargo --version 2>$null) -replace "cargo\s+", ""
  $manifest = [ordered]@{
    app_name     = $AppName
    installed_at = (Get-Date).ToUniversalTime().ToString("o")
    install_mode = $InstallMode
    source_repo  = $RepoRoot
    cargo_version = $cargoVersion
  }

  $manifest | ConvertTo-Json | Set-Content -LiteralPath (Get-ManifestPath -Root $AppRoot) -Encoding UTF8
}

function Write-Launcher {
  param(
    [string]$AppRoot,
    [string]$LauncherPath
  )

  $launcherDir = Split-Path -Parent $LauncherPath
  New-Item -ItemType Directory -Path $launcherDir -Force | Out-Null

  $content = @"
@echo off
setlocal
set "APP_ROOT=$AppRoot"
set "BINARY=%APP_ROOT%\bin\$AppName.exe"
if not exist "%BINARY%" (
  echo [FAIL] NailSnake is not installed correctly at "%APP_ROOT%".
  exit /b 1
)
"%BINARY%" %*
"@

  Set-Content -LiteralPath $LauncherPath -Value $content -Encoding ASCII
}

function Add-UserPathEntry {
  param([string]$Directory)

  New-Item -ItemType Directory -Path $Directory -Force | Out-Null

  $current = [Environment]::GetEnvironmentVariable("Path", "User")
  $entries = @()
  if (-not [string]::IsNullOrWhiteSpace($current)) {
    $entries = $current.Split(";") | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
  }

  $alreadyPresent = $false
  foreach ($entry in $entries) {
    if ($entry.TrimEnd("\") -ieq $Directory.TrimEnd("\")) {
      $alreadyPresent = $true
      break
    }
  }

  if (-not $alreadyPresent) {
    $newPath = (($entries + $Directory) | Select-Object -Unique) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    if ($env:Path -notlike "*$Directory*") {
      $env:Path = "$Directory;$env:Path"
    }
    Write-Success "Added $Directory to the user PATH."
  } else {
    Write-Info "$Directory is already present in the user PATH."
  }
}

function Remove-UserPathEntry {
  param([string]$Directory)

  $current = [Environment]::GetEnvironmentVariable("Path", "User")
  if ([string]::IsNullOrWhiteSpace($current)) {
    return
  }

  $entries = $current.Split(";") | Where-Object {
    -not [string]::IsNullOrWhiteSpace($_) -and $_.TrimEnd("\") -ine $Directory.TrimEnd("\")
  }

  [Environment]::SetEnvironmentVariable("Path", ($entries -join ";"), "User")
}

function Invoke-SmokeTest {
  param([string]$LauncherPath)

  Write-Info "Running smoke test."
  & cmd.exe /c "`"$LauncherPath`" --version" *> $null
  if ($LASTEXITCODE -ne 0) {
    Stop-Step "Smoke test failed for $LauncherPath."
  }
  Write-Success "Smoke test passed."
}

function Show-InstallSummary {
  param(
    [string]$InstallMode,
    [string]$AppRoot,
    [string[]]$LauncherPaths
  )

  Write-Success "NailSnake $InstallMode completed successfully."
  Write-Info "Installed application: $AppRoot"
  Write-Info "System command: $AppName"
  Write-Info "Launcher paths: $($LauncherPaths -join ', ')"
  Write-Info "Man page: $AppRoot\man\man1\$AppName.1"
}

function Install-Application {
  param(
    [string]$TargetRoot,
    [string[]]$LauncherPaths,
    [string]$InstallMode,
    [switch]$AddToPath
  )

  Confirm-ManagedInstallTarget -Root $TargetRoot
  Confirm-RustInstalled

  $parent = Split-Path -Parent $TargetRoot
  $stageRoot = Join-Path $parent ".$AppName-staging-$PID"
  $stageBin = Join-Path $stageRoot "bin"
  $backupRoot = Join-Path $parent ".$AppName-backup-$PID"
  $restored = $false

  $null = Remove-TreeSafe -PathToRemove $stageRoot
  $null = Remove-TreeSafe -PathToRemove $backupRoot
  New-Item -ItemType Directory -Path $stageBin -Force | Out-Null

  try {
    Write-Info "Preparing staged files for $InstallMode."
    Build-Release
    Copy-Binary -DestDir $stageBin
    Write-Manifest -AppRoot $stageRoot -InstallMode $InstallMode
    Install-ManPage -AppRoot $stageRoot

    if (Test-Path -LiteralPath $TargetRoot) {
      Move-Item -LiteralPath $TargetRoot -Destination $backupRoot -Force
    }

    Move-Item -LiteralPath $stageRoot -Destination $TargetRoot -Force
    Write-Launcher -AppRoot $TargetRoot -LauncherPath $LauncherPaths[0]

    if ($AddToPath) {
      Add-UserPathEntry -Directory $UserBinDir
    }

    Invoke-SmokeTest -LauncherPath $LauncherPaths[0]

    $null = Remove-TreeSafe -PathToRemove $backupRoot
    Show-InstallSummary -InstallMode $InstallMode -AppRoot $TargetRoot -LauncherPaths $LauncherPaths
  } catch {
    $originalError = $_.Exception.Message
    Write-WarningLine "Attempting to restore the previous installation state."
    if (Test-Path -LiteralPath $backupRoot) {
      if (Test-Path -LiteralPath $TargetRoot) {
        $null = Remove-TreeSafe -PathToRemove $TargetRoot
      }

      if (-not (Test-Path -LiteralPath $TargetRoot)) {
        Move-Item -LiteralPath $backupRoot -Destination $TargetRoot -Force
        $restored = $true
      }
    }

    $null = Remove-TreeSafe -PathToRemove $stageRoot
    if (-not $restored) {
      $null = Remove-TreeSafe -PathToRemove $backupRoot
    }

    Stop-Step $originalError
  }
}

function Uninstall-Application {
  if (-not (Test-Path -LiteralPath $InstallRoot)) {
    Write-WarningLine "No managed installation was found at $InstallRoot."
  } elseif (-not (Test-ManagedInstall -Root $InstallRoot)) {
    Stop-Step "Refusing to remove $InstallRoot because it is not marked as managed by this installer."
  } else {
    Remove-TreeSafe -PathToRemove $InstallRoot
    Write-Success "Removed $InstallRoot"
  }

  if (Test-Path -LiteralPath $SystemLauncher) {
    Remove-Item -LiteralPath $SystemLauncher -Force
    Write-Success "Removed launcher $SystemLauncher"
  }

  Remove-UserPathEntry -Directory $UserBinDir
  Write-Success "User PATH cleaned up."
}

switch ($Mode) {
  "install" {
    Install-Application -TargetRoot $InstallRoot -LauncherPaths @($SystemLauncher) -InstallMode "install" -AddToPath
  }
  "test" {
    Confirm-RustInstalled
    Build-Debug
    Invoke-Tests
    Install-Application -TargetRoot $TestRoot -LauncherPaths @($TestLauncher) -InstallMode "test"
    Write-Info "Repo-local test binary: $TestLauncher"
  }
  "update" {
    Confirm-RustInstalled
    if (-not (Test-ManagedInstall -Root $InstallRoot)) {
      Stop-Step "No managed installation was found to update. Run install first."
    }
    Install-Application -TargetRoot $InstallRoot -LauncherPaths @($SystemLauncher) -InstallMode "update" -AddToPath
  }
  "uninstall" {
    Uninstall-Application
  }
  "build" {
    Confirm-RustInstalled
    Build-Release
    Write-Success "Release binary built at target\release\$AppName.exe"
  }
  "check" {
    Confirm-RustInstalled
    & cargo check
    if ($LASTEXITCODE -eq 0) {
      Write-Success "Code check passed."
    } else {
      Stop-Step "Code check failed."
    }
  }
  "run" {
    Confirm-RustInstalled
    & cargo run -- $Remaining
  }
  default {
    Stop-Step "Unsupported mode: $Mode"
  }
}
