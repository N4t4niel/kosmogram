# Builds signed installers for x64 and ARM64 and latest.json for the updater; with -Publish creates the GitHub release.
#
#   powershell -File tools\release.ps1 -Notes "Co je nového" [-Publish]
#
# Needs: Tauri CLI 2 (tauri.cmd on PATH or -Tauri), gh logged in, the private key and its password.
# The key stays outside the repository (default C:\Users\<user>\.tauri\kosmogram.key). Its password is
# stored with Windows DPAPI in kosmogram.key.password.dpapi, readable only by this user on this computer.
#
# Every release also carries Kosmogram-x64-setup.exe and Kosmogram-arm64-setup.exe, so these links always
# lead to the newest version:
#   https://github.com/<repo>/releases/latest/download/Kosmogram-x64-setup.exe
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
$out = "$app\dist\$tag"
New-Item -ItemType Directory -Force $out | Out-Null

$secure = Get-Content "$Key.password.dpapi" | ConvertTo-SecureString
$env:TAURI_SIGNING_PRIVATE_KEY = $Key
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure))

# target, name in the setup file from Tauri, stable name, updater platform
$targets = @(
  @("x86_64-pc-windows-msvc", "x64", "windows-x86_64"),
  @("aarch64-pc-windows-msvc", "arm64", "windows-aarch64")
)
$platforms = [ordered]@{}
$assets = @()
try {
  foreach ($t in $targets) {
    $target, $arch, $platform = $t
    Push-Location "$app\desktop"
    # Tauri writes progress to stderr, which Windows PowerShell would treat as an error; the exit code decides.
    $ErrorActionPreference = "Continue"
    & $Tauri build --target $target 2>&1 | ForEach-Object { "$_" }
    $code = $LASTEXITCODE
    $ErrorActionPreference = "Stop"
    Pop-Location
    if ($code) { throw "tauri build pro $target selhal" }
    $setup = "$($conf.productName)_${version}_${arch}-setup.exe"
    $bundle = "$app\target\$target\release\bundle\nsis"
    Copy-Item "$bundle\$setup", "$bundle\$setup.sig" $out -Force
    Copy-Item "$bundle\$setup" "$out\$($conf.productName)-$arch-setup.exe" -Force
    $platforms[$platform] = @{
      signature = (Get-Content "$out\$setup.sig" -Raw).Trim()
      url = "https://github.com/$Repo/releases/download/$tag/$setup"
    }
    $assets += "$out\$setup", "$out\$($conf.productName)-$arch-setup.exe"
  }
}
finally { Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY, Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue }

$latest = [ordered]@{
  version = $version
  notes = $Notes
  pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
  platforms = $platforms
}
[IO.File]::WriteAllText("$out\latest.json", ($latest | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
$assets += "$out\latest.json"
Write-Host "Připraveno v $out"

if ($Publish) {
  gh release create $tag @assets --repo $Repo --title "Kosmogram $($version -replace '\.0$','')" --notes $Notes
  if ($LASTEXITCODE) { throw "gh release create selhal" }
  Write-Host "Vydání $tag je na https://github.com/$Repo/releases/tag/$tag"
}
