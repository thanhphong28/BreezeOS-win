param(
  [string]$Version = "",
  [switch]$NoBuild,
  [switch]$AutoFromHook
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$toml = Get-Content (Join-Path $root "Cargo.toml") -Raw
if (-not $Version) {
  if ($toml -match 'version\s*=\s*"([^"]+)"') {
    $Version = $Matches[1]
  } else {
    throw "Could not read version from Cargo.toml"
  }
}

$tag = "v$Version"
$repo = "thanhphong28/BreezeOS-win"

$credExe = "C:\Users\User\AppData\Local\MinGit\mingw64\bin\git-credential-wincred.exe"
if (-not (Test-Path $credExe)) {
  $credExe = "git-credential-wincred.exe"
}
$lines = "protocol=https`nhost=github.com`n`n" | & $credExe get
$token = ($lines | Where-Object { $_ -like "password=*" }) -replace "^password=", ""
if (-not $token) {
  throw "No GitHub token found in git-credential-wincred"
}

$headers = @{
  Authorization = "Bearer $token"
  Accept        = "application/vnd.github+json"
  "User-Agent"  = "BreezeOS-Release"
}

try {
  $existing = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/tags/$tag" -Headers $headers -ErrorAction Stop
  if ($existing -and $AutoFromHook) {
    Write-Host "Release $tag already exists on GitHub. Skipping auto-release."
    exit 0
  }
} catch {
  $existing = $null
}

$dist = Join-Path $root "dist"
$setupExe = Join-Path $dist "BreezeOS-Setup-$Version.exe"
$portableZip = Join-Path $dist "BreezeOS-$Version-win-x64.zip"

if (-not $NoBuild -or -not (Test-Path $setupExe) -or -not (Test-Path $portableZip)) {
  & (Join-Path $PSScriptRoot "build-installer.ps1") -Version $Version -NoBuild:$NoBuild
}

if ($existing) {
  $release = $existing
  Write-Host "Updating existing release $tag..."
} else {
  Write-Host "Creating GitHub Release $tag (BreezeOS $Version)..."
  $body = @{
    tag_name               = $tag
    target_commitish       = "main"
    name                   = "BreezeOS $Version"
    body                   = "## BreezeOS $Version`n`n- **Installer**: ``BreezeOS-Setup-$Version.exe`` (recommended, supports automatic in-app updates)`n- **Portable**: ``BreezeOS-$Version-win-x64.zip```n- **Languages**: English (default) & Vietnamese (Tiếng Việt)"
    draft                  = $false
    prerelease             = $false
    generate_release_notes = $true
  } | ConvertTo-Json
  $release = Invoke-RestMethod -Method Post -Uri "https://api.github.com/repos/$repo/releases" -Headers $headers -Body ([Text.Encoding]::UTF8.GetBytes($body)) -ContentType "application/json; charset=utf-8"
}

$uploadBase = ($release.upload_url -replace "\{\?name,label\}$", "")

foreach ($filePath in @($setupExe, $portableZip)) {
  $fileName = Split-Path $filePath -Leaf
  foreach ($asset in $release.assets) {
    if ($asset.name -eq $fileName) {
      Invoke-RestMethod -Method Delete -Uri "https://api.github.com/repos/$repo/releases/assets/$($asset.id)" -Headers $headers | Out-Null
    }
  }
  Write-Host "Uploading $fileName..."
  $contentType = if ($fileName.EndsWith(".exe")) { "application/vnd.microsoft.portable-executable" } else { "application/zip" }
  $uploaded = Invoke-RestMethod -Method Post -Uri "${uploadBase}?name=$fileName" -Headers $headers -InFile $filePath -ContentType $contentType
  Write-Host "Uploaded: $($uploaded.browser_download_url)"
}

Write-Host "Release $tag is live at: $($release.html_url)"
