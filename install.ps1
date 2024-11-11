# deploy_env_check.ps1

# Check for Rust installation
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust is not installed. Installing Rust..."
    Invoke-Expression "& { $(Invoke-WebRequest -Uri https://sh.rustup.rs -UseBasicParsing).Content } -y"
} else {
    Write-Host "Rust is already installed."
}

# Check for OpenCV
$opencvInstalled = (Get-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*' |
    Where-Object { $_.DisplayName -match "OpenCV" })
if (-not $opencvInstalled) {
    Write-Host "OpenCV is not installed. Please install OpenCV manually for Rust compatibility."
} else {
    Write-Host "OpenCV is already installed."
}

# Check for Python (if additional processing or model setup is required)
if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "Python is not installed. Installing Python..."
    winget install -e --id Python.Python.3.9
} else {
    Write-Host "Python is already installed."
}

# Check for required Rust dependencies
Write-Host "Verifying Rust dependencies..."
Invoke-Expression "cargo install opencv glob tokio" -ErrorAction Stop
Write-Host "Dependencies installed successfully."

# Run environment test
Write-Host "Running environment test..."
try {
    cargo build --release
    Write-Host "Environment is ready for deployment."
} catch {
    Write-Host "Error encountered during setup. Check your configuration."
}

Write-Host "Environment setup complete. Ready to process screenshots."
