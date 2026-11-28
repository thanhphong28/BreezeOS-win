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

$shaSetup = (Get-FileHash $setupExe -Algorithm SHA256).Hash.ToLower()
$shaZip   = (Get-FileHash $portableZip -Algorithm SHA256).Hash.ToLower()

$notes = @"
<p align="center">
  <img src="https://raw.githubusercontent.com/thanhphong28/BreezeOS-win/main/assets/logo.png" alt="BreezeOS" width="140">
</p>

## ✨ BreezeOS $Version

Giao diện kính mờ hiện đại cho **Windows 11** — phát triển bởi **Nguyễn Thanh Phong (Breeze Boy)**, viết thuần bằng **Rust** (~15 MB RAM khi chạy nền), hỗ trợ đầy đủ **Tiếng Việt** & **English**.

### 📦 Tệp tải xuống
- **Bộ cài đặt khuyên dùng**: ``BreezeOS-Setup-$Version.exe`` *(cài đặt không cần quyền Admin, hỗ trợ tự động cập nhật ngay trong ứng dụng)*
- **Bản chạy ngay (Portable)**: ``BreezeOS-$Version-win-x64.zip`` *(giải nén và chạy trực tiếp ``breezeos.exe``)*

### 🔐 Mã kiểm tra SHA-256
| Tệp | SHA-256 |
|---|---|
| ``BreezeOS-Setup-$Version.exe`` | ``$shaSetup`` |
| ``BreezeOS-$Version-win-x64.zip`` | ``$shaZip`` |
"@

if ($existing) {
  $patchBody = @{
    name       = "BreezeOS $Version"
    body       = $notes
    draft      = $false
    prerelease = $false
  } | ConvertTo-Json
  $release = Invoke-RestMethod -Method Patch -Uri "https://api.github.com/repos/$repo/releases/$($existing.id)" -Headers $headers -Body ([Text.Encoding]::UTF8.GetBytes($patchBody)) -ContentType "application/json; charset=utf-8"
  Write-Host "Updated existing release $tag..."
} else {
  Write-Host "Creating GitHub Release $tag (BreezeOS $Version)..."
  $body = @{
    tag_name               = $tag
    target_commitish       = "main"
    name                   = "BreezeOS $Version"
    body                   = $notes
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

# Update release-meta/latest.json so in-app updater immediately resolves the new version and SHA-256
$metaObj = @{
  tag_name   = $tag
  html_url   = "https://github.com/$repo/releases/tag/$tag"
  draft      = $false
  prerelease = $false
  assets     = @(
    @{
      name                 = "BreezeOS-Setup-$Version.exe"
      browser_download_url = "https://github.com/$repo/releases/download/$tag/BreezeOS-Setup-$Version.exe"
      digest               = "sha256:$shaSetup"
    },
    @{
      name                 = "BreezeOS-$Version-win-x64.zip"
      browser_download_url = "https://github.com/$repo/releases/download/$tag/BreezeOS-$Version-win-x64.zip"
      digest               = "sha256:$shaZip"
    }
  )
}
$metaJson = $metaObj | ConvertTo-Json -Depth 5
$b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($metaJson))

try {
  $existingMeta = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/contents/latest.json?ref=release-meta" -Headers $headers -ErrorAction Stop
  $putBody = @{
    message = "chore: update latest.json for $tag"
    content = $b64
    branch  = "release-meta"
    sha     = $existingMeta.sha
  } | ConvertTo-Json
} catch {
  $putBody = @{
    message = "chore: publish latest.json for $tag"
    content = $b64
    branch  = "release-meta"
  } | ConvertTo-Json
}
Invoke-RestMethod -Method Put -Uri "https://api.github.com/repos/$repo/contents/latest.json" -Headers $headers -Body ([Text.Encoding]::UTF8.GetBytes($putBody)) -ContentType "application/json; charset=utf-8" | Out-Null
Write-Host "Updated CDN metadata on branch release-meta."
Write-Host "Release $tag is live at: $($release.html_url)"
