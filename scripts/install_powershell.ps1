$ErrorActionPreference = "Stop"

$Repo = "zhangzhenxiang666/env-manager"
$InstallDir = "$env:USERPROFILE\.config\env-manage\bin"
$TargetBin = "$InstallDir\env-manage.exe"

Write-Host "Fetching latest version..."
try {
    # Attempt to fetch the latest release page with 0 redirects to get the location header
    $Response = Invoke-WebRequest -Uri "https://github.com/$Repo/releases/latest" -Method Head -MaximumRedirection 0 -ErrorAction Stop
    $LatestTagUrl = $Response.Headers.Location
} catch {
    # PowerShell throws an error on 3xx redirects when MaximumRedirection is 0, which is expected
    if ($_.Exception.Response.Headers["Location"]) {
        $LatestTagUrl = $_.Exception.Response.Headers["Location"]
    } else {
        Write-Error "Could not determine latest version: $($_.Exception.Message)"
        exit 1
    }
}

# Extract tag from URL (e.g., https://github.com/user/repo/releases/tag/v1.0.0)
$LatestTag = $LatestTagUrl | Split-Path -Leaf

if ([string]::IsNullOrEmpty($LatestTag)) {
    Write-Error "Could not determine latest version tag from URL: $LatestTagUrl"
    exit 1
}

Write-Host "Latest version: $LatestTag"
$AssetName = "env-manage-$LatestTag-windows-amd64.zip"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$AssetName"
$TempZip = "$env:TEMP\$AssetName"
$TempDir = "$env:TEMP\env-manage-install"

Write-Host "Downloading from: $DownloadUrl"
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip

Write-Host "Extracting..."
if (Test-Path $TempDir) { Remove-Item -Recurse -Force $TempDir }
Expand-Archive -Path $TempZip -DestinationPath $TempDir -Force

# Find the binary (it's inside a folder in the zip)
$FoundBin = Get-ChildItem -Path $TempDir -Recurse -Filter "env-manage.exe" | Select-Object -First 1

if ($null -eq $FoundBin) {
    Write-Error "Could not find env-manage.exe in the downloaded archive."
    exit 1
}

# Install
if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null }
Move-Item -Path $FoundBin.FullName -Destination $TargetBin -Force

# Cleanup
Remove-Item -Force $TempZip
Remove-Item -Recurse -Force $TempDir

Write-Host "Installed to $TargetBin"

# Configure Profile
$ProfilePath = $PROFILE 
if (-not (Test-Path $ProfilePath)) {
    New-Item -ItemType File -Path $ProfilePath -Force | Out-Null
}

$InitCmd = "Invoke-Expression (& ""$TargetBin"" init powershell | Out-String)"

if (Get-Content $ProfilePath | Select-String "env-manage init powershell") {
    Write-Host "Profile already configured."
} else {
    Add-Content -Path $ProfilePath -Value "`n# env-manage"
    Add-Content -Path $ProfilePath -Value $InitCmd
    Write-Host "Added init command to $ProfilePath"
}

Write-Host "Installation complete! Please restart your PowerShell session."
