param(
    [ValidateSet("Prompt", "User", "Machine")]
    [string]$Scope = "Prompt"
)

$ErrorActionPreference = "Stop"

$AppName = "nailsnake"
$DisplayName = "NailSnake"
$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..")
$CargoToml = Join-Path $RootDir "Cargo.toml"
$ExePath = Join-Path $RootDir "target\release\nailsnake.exe"

function Get-CargoVersion {
    param([string]$ManifestPath)
    $match = Select-String -Path $ManifestPath -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if ($null -eq $match) {
        throw "Could not read the package version from $ManifestPath."
    }
    return $match.Matches[0].Groups[1].Value
}

$Version = Get-CargoVersion -ManifestPath $CargoToml

function Ask-Choice {
    param(
        [string]$Question,
        [string[]]$Choices,
        [string]$Default
    )

    Write-Host $Question
    for ($i = 0; $i -lt $Choices.Count; $i++) {
        Write-Host "  $($i + 1)) $($Choices[$i])"
    }
    $answer = Read-Host "Selection [$Default]"
    if ([string]::IsNullOrWhiteSpace($answer)) {
        return $Default
    }
    if ($answer -match '^\d+$') {
        $index = [int]$answer - 1
        if ($index -ge 0 -and $index -lt $Choices.Count) {
            return $Choices[$index]
        }
    }
    return $answer
}

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Restart-AsAdmin {
    param([string]$RequestedScope)
    $args = @(
        "-NoProfile",
        "-ExecutionPolicy", "Bypass",
        "-File", "`"$PSCommandPath`"",
        "-Scope", $RequestedScope
    )
    Start-Process -FilePath "powershell.exe" -ArgumentList $args -Verb RunAs
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required. Install Rust from https://rustup.rs/ and run this script again."
}

if ($Scope -eq "Prompt") {
    $Scope = Ask-Choice `
        -Question "Install $DisplayName for this user or the full PC?" `
        -Choices @("User", "Machine") `
        -Default "User"
}

if ($Scope -eq "Machine" -and -not (Test-Admin)) {
    Write-Host "Machine-wide install needs administrator privileges. Requesting elevation..."
    Restart-AsAdmin -RequestedScope "Machine"
    exit 0
}

Write-Host "Building $DisplayName release binary..."
Push-Location $RootDir
try {
    cargo build --release --locked --verbose
} finally {
    Pop-Location
}

if ($Scope -eq "Machine") {
    $InstallDir = Join-Path $env:ProgramFiles $DisplayName
    $PathTarget = [EnvironmentVariableTarget]::Machine
} else {
    $ProgramsRoot = Join-Path $env:LOCALAPPDATA "Programs"
    $InstallDir = Join-Path $ProgramsRoot $DisplayName
    $PathTarget = [EnvironmentVariableTarget]::User
}

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Copy-Item -Path $ExePath -Destination (Join-Path $InstallDir "$AppName.exe") -Force
Copy-Item -Path (Join-Path $RootDir "README.md") -Destination (Join-Path $InstallDir "README.md") -Force
Copy-Item -Path (Join-Path $RootDir "LICENSE") -Destination (Join-Path $InstallDir "LICENSE") -Force
$ManDir = Join-Path $InstallDir "share\man\man1"
New-Item -ItemType Directory -Path $ManDir -Force | Out-Null
Copy-Item -Path (Join-Path $RootDir "man\nailsnake.1") -Destination (Join-Path $ManDir "nailsnake.1") -Force

$CurrentPath = [Environment]::GetEnvironmentVariable("Path", $PathTarget)
$PathParts = @()
if (-not [string]::IsNullOrWhiteSpace($CurrentPath)) {
    $PathParts = $CurrentPath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
}

if ($PathParts -notcontains $InstallDir) {
    $NewPath = (($PathParts + $InstallDir) -join ';')
    [Environment]::SetEnvironmentVariable("Path", $NewPath, $PathTarget)
    $env:Path = (($env:Path -split ';') + $InstallDir | Select-Object -Unique) -join ';'
}

Write-Host "$DisplayName $Version installed to: $InstallDir"
Write-Host "Environment updated for $Scope scope:"
Write-Host "  PATH includes $InstallDir"
Write-Host "  Man page: $(Join-Path $ManDir 'nailsnake.1')"
Write-Host ""
Write-Host "Open a new cmd.exe or PowerShell window, then run: $AppName"
