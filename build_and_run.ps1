# PowerShell Script to Automate DLL and Rust Build Process

$ErrorActionPreference = "Stop"

# Paths (Change if needed)
$ProjectRoot = Get-Location
$TargetDir = "$ProjectRoot\target\debug"
$GoLibDir = "$ProjectRoot\go_lib"   # Path where `go_lib.dll` and `go_lib.def` are located
$DefFile = "$GoLibDir\go_lib.def"
$DllFile = "$GoLibDir\go_lib.dll"
$LibFile = "$GoLibDir\go_lib.lib"

# Step 1: Generate `go_lib.lib` using dlltool
Write-Host "Generating go_lib.lib..."
if (Test-Path $DefFile) {
    dlltool -d $DefFile -D go_lib.dll -l $LibFile
    Write-Host "go_lib.lib generated successfully!"
}
else {
    Write-Host "Error: $DefFile not found!" -ForegroundColor Red
    exit 1
}

# Step 2: Copy DLL to target directory
Write-Host "Copying go_lib.dll to Rust target directory..."
Copy-Item -Force $DllFile $TargetDir

# Step 3: Temporarily add DLL directory to PATH
$env:PATH += ";$GoLibDir"
Write-Host "Added DLL directory to PATH."

# Step 4: Build and Run Rust project
Write-Host "Building Rust project..."
cargo build

if ($?) {
    Write-Host "Running Rust project..."
    cargo run
}
else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
