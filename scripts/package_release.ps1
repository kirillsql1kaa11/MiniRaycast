param(
    [string]$OutputDir = "release-portable"
)

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir/scripts" | Out-Null
Copy-Item "src-tauri/target/release/miniraycast.exe" -Destination "$OutputDir/MiniRaycast.exe" -Force
Copy-Item "plugins" -Destination "$OutputDir/plugins" -Recurse -Force
Copy-Item "scripts/system_controls.ps1" -Destination "$OutputDir/scripts/system_controls.ps1" -Force

Write-Host "Portable package created in $OutputDir"
