# Starts the game: builds whatever is out of date, then starts the host, which opens the browser.
# Run it through play.cmd (double-click it, or `play` in a terminal). Arguments go to fh-host,
# e.g. `play --fresh`, `play --port 5501` or `play --no-browser`.

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Fail($message) {
    Write-Host ""
    Write-Host $message -ForegroundColor Red
    exit 1
}

function Step($message) {
    Write-Host ""
    Write-Host "== $message" -ForegroundColor Cyan
}

# Arguments go to the host (it opens the browser itself unless given --no-browser). The port is
# needed to check whether it's already running.
$hostArgs = @($args)
$openBrowser = -not ($hostArgs -contains "--no-browser")
$port = 5500
for ($i = 0; $i -lt $args.Count - 1; $i++) {
    if ($args[$i] -eq "--port") { $port = [int]$args[$i + 1] }
}
$url = "http://localhost:$port/"

function HostRunning {
    try {
        $info = Invoke-RestMethod -Uri "$($url)api/info" -TimeoutSec 1
        return $info.app -eq "fh-host"
    } catch {
        return $false
    }
}

if (HostRunning) {
    Write-Host "The game is already running at $url"
    if ($openBrowser) { Start-Process $url }
    exit 0
}

foreach ($tool in "cargo", "npm") {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        Fail "Can't find '$tool'. Install Rust (https://rustup.rs) and Node.js (https://nodejs.org), then try again."
    }
}

# The web app (and its WebAssembly engine) is rebuilt when any of its sources is newer than the build.
$dist = Join-Path $root "web\dist\index.html"
if (-not (Test-Path (Join-Path $root "web\node_modules"))) {
    Step "Installing the web app's packages (first run only)"
    Push-Location web
    & npm ci
    $ok = $LASTEXITCODE -eq 0
    Pop-Location
    if (-not $ok) { Fail "Installing packages failed (see above)." }
}
$sources = @("web\src", "web\public", "web\index.html", "web\package.json", "web\vite.config.ts",
             "crates\core\src", "crates\ai\src", "crates\ai\models", "crates\wasm\src", "Cargo.toml")
$newest = Get-ChildItem -Path $sources -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notlike "*\engine\pkg\*" } |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not (Test-Path $dist) -or ($newest -and $newest.LastWriteTime -gt (Get-Item $dist).LastWriteTime)) {
    Step "Building the web app (a minute or two)"
    Push-Location web
    & npm run build
    $ok = $LASTEXITCODE -eq 0
    Pop-Location
    if (-not $ok) { Fail "Building the web app failed (see above)." }
}

Step "Building the host (the first time takes a few minutes)"
& cargo build --release -p fh-host
if ($LASTEXITCODE -ne 0) { Fail "Building the host failed (see above)." }

Step "Starting the game"
$exe = Join-Path $root "target\release\fh-host.exe"
& $exe @hostArgs
exit $LASTEXITCODE
