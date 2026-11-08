# Publishes the web app to Cloudflare Pages (free), at https://<project>.pages.dev, so it can be
# added to an iPhone's home screen and played anywhere, offline included (the engine runs on the
# phone; the service worker keeps every file). Builds web/dist first.
#
#   publish.cmd                     (project "fivehundred")
#   publish.cmd -Project my-500     (another name; it becomes the address)
#
# The first run opens a browser to sign in to Cloudflare (a free account), and creates the project.
# Wifi play with friends still needs the laptop (play.cmd); the published copy is for solo play.

param(
    [string]$Project = "fivehundred"
)

# Not "Stop": Windows PowerShell treats a program's stderr output as an error when it's captured.
# Each step checks the program's exit code instead.
$root = Split-Path -Parent $PSScriptRoot
$web = Join-Path $root "web"
# Pinned, so a new release can't change what this does.
$wrangler = @("--yes", "wrangler@4.151.2")

function Wrangler {
    & npx @wrangler @args
    if ($LASTEXITCODE -ne 0) { throw "wrangler $($args -join ' ') failed" }
}

Push-Location $web
try {
    Write-Host "Building the app..."
    & npm run build
    if ($LASTEXITCODE -ne 0) { throw "the build failed" }

    # Signed in? If not, Cloudflare's sign-in page opens in the browser.
    $whoami = (& npx @wrangler whoami 2>&1) -join "`n"
    if ($whoami -match "not authenticated") {
        Write-Host "Sign in to Cloudflare in the browser window that opens..."
        Wrangler login
    }

    $projects = (& npx @wrangler pages project list 2>&1) -join "`n"
    if ($projects -notmatch "(?m)\b$([regex]::Escape($Project))\b") {
        Write-Host "Creating the Pages project '$Project'..."
        Wrangler pages project create $Project --production-branch main
    }

    Write-Host "Uploading..."
    Wrangler pages deploy dist --project-name $Project --branch main --commit-dirty=true

    Write-Host ""
    Write-Host "Published. On your iPhone, open https://$Project.pages.dev in Safari (if that name was"
    Write-Host "taken, Cloudflare shows the actual address above and in its dashboard), then tap Share,"
    Write-Host "then Add to Home Screen. Open it once from the icon while online; after that it works offline."
    Write-Host "Run publish.cmd again after changes: the app updates the next time it's opened online."
}
finally {
    Pop-Location
}
