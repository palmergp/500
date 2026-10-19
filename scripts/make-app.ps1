# Builds the single-file game: app\500.exe, with the web app inside it. Copy that one file to any
# Windows PC and double-click it to play (nothing else to install). Run it through make-app.cmd.

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Fail($message) {
    Write-Host ""
    Write-Host $message -ForegroundColor Red
    exit 1
}

foreach ($tool in "cargo", "npm") {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        Fail "Can't find '$tool'. Install Rust (https://rustup.rs) and Node.js (https://nodejs.org), then try again."
    }
}

Write-Host "== Building the web app" -ForegroundColor Cyan
Push-Location web
if (-not (Test-Path node_modules)) {
    & npm ci
    if ($LASTEXITCODE -ne 0) { Pop-Location; Fail "Installing packages failed (see above)." }
}
& npm run build
$ok = $LASTEXITCODE -eq 0
Pop-Location
if (-not $ok) { Fail "Building the web app failed (see above)." }

# Its own target folder, so it doesn't fight play.cmd's build (which reads web/dist from disk).
# The C runtime is built in (crt-static), so the program runs on a PC without Visual C++ installed.
Write-Host ""
Write-Host "== Building 500.exe (a few minutes the first time)" -ForegroundColor Cyan
$env:RUSTFLAGS = "-C target-feature=+crt-static"
& cargo build --release -p fh-host --features embed --target-dir target/app
Remove-Item Env:RUSTFLAGS
if ($LASTEXITCODE -ne 0) { Fail "Building the program failed (see above)." }

New-Item -ItemType Directory -Force app | Out-Null
Copy-Item target\app\release\fh-host.exe app\500.exe -Force
$size = [math]::Round((Get-Item app\500.exe).Length / 1MB, 1)
Write-Host ""
Write-Host "Done: app\500.exe ($size MB)." -ForegroundColor Green
Write-Host "Copy that one file to any Windows PC and double-click it to play."
