# Builds the signed installer and latest.json for the updater; with -Publish creates the GitHub release.
#
#   powershell -File tools\release.ps1 -Notes "Co je nového" [-Publish]
#
# Needs: Tauri CLI 2 (tauri.cmd on PATH or -Tauri), gh logged in, the private key and its password
# (default C:\Users\<user>\.tauri\kosmogram.key and .key.password, never in the repository).
param(
  [string]$Notes = "",
  [switch]$Publish,
  [string]$Tauri = "tauri",
  [string]$Key = "$env:USERPROFILE\.tauri\kosmogram.key",
  [string]$Repo = "N4t4niel/kosmogram"
)
$ErrorActionPreference = "Stop"
$app = Split-Path $PSScriptRoot -Parent
$conf = Get-Content "$app\desktop\tauri.conf.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$version = $conf.version
$tag = "v$version"
$setup = "$($conf.productName)_${version}_x64-setup.exe"
$bundle = "$app\target\x86_64-pc-windows-msvc\release\bundle\nsis"

$env:TAURI_SIGNING_PRIVATE_KEY = $Key
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (Get-Content "$Key.password" -Raw).Trim()
Push-Location "$app\desktop"
# Tauri writes progress to stderr, which Windows PowerShell would treat as an error; the exit code decides.
$ErrorActionPreference = "Continue"
try { & $Tauri build --target x86_64-pc-windows-msvc 2>&1 | ForEach-Object { "$_" }; $code = $LASTEXITCODE }
finally { Pop-Location; Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY, Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD }
$ErrorActionPreference = "Stop"
if ($code) { throw "tauri build selhal" }

$out = "$app\dist\$tag"
New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item "$bundle\$setup", "$bundle\$setup.sig" $out -Force
$latest = [ordered]@{
  version = $version
  notes = $Notes
  pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
  platforms = @{ "windows-x86_64" = @{
    signature = (Get-Content "$out\$setup.sig" -Raw).Trim()
    url = "https://github.com/$Repo/releases/download/$tag/$setup"
  } }
}
[IO.File]::WriteAllText("$out\latest.json", ($latest | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
Write-Host "Připraveno v $out"

if ($Publish) {
  gh release create $tag "$out\$setup" "$out\latest.json" --repo $Repo --title "Kosmogram $($version -replace '\.0$','')" --notes $Notes
  if ($LASTEXITCODE) { throw "gh release create selhal" }
  Write-Host "Vydání $tag je na https://github.com/$Repo/releases/tag/$tag"
}
