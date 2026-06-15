# Install NailSnake man page on Windows (Git Bash / MSYS2 / WSL-style layouts).
# Native Windows has no man-db; use Git Bash "man" or WSL after install.
param(
    [string]$Prefix = "$env:ProgramFiles\NailSnake",
    [switch]$UserLocal
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$ManSrc = Join-Path $Root "man\nailsnake.1"

if (-not (Test-Path $ManSrc)) {
    Write-Error "Man page not found: $ManSrc"
}

if ($UserLocal) {
    $ManDir = Join-Path $env:USERPROFILE "scoop\persist\man\man1"
    if (-not (Test-Path (Split-Path $ManDir))) {
        $ManDir = Join-Path $env:USERPROFILE ".local\share\man\man1"
    }
} else {
    $ManDir = Join-Path $Prefix "share\man\man1"
}

New-Item -ItemType Directory -Force -Path $ManDir | Out-Null
Copy-Item -Force $ManSrc (Join-Path $ManDir "nailsnake.1")

Write-Host "Installed: $(Join-Path $ManDir 'nailsnake.1')"
Write-Host ""
Write-Host "View with Git Bash:  man nailsnake"
Write-Host "Or set MANPATH and use man from MSYS2/WSL."
