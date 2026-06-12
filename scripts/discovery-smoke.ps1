param(
    [int]$Seconds = 3,
    [int]$ServicePort = 42424,
    [int]$UdpPort = 42425
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$tauriDir = Join-Path $repo "src-tauri"
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"

if (-not (Test-Path $cargo)) {
    $cargo = "cargo"
}

Push-Location $tauriDir
try {
    & $cargo run --example discovery_smoke -- --seconds $Seconds --service-port $ServicePort --udp-port $UdpPort
} finally {
    Pop-Location
}
