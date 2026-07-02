# PowerShell script to run the Rust IAM Platform project

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "         Rust IAM Platform Startup Script" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

# 1. Update PATH for the current session to include Cargo
$CargoBinPath = "$env:USERPROFILE\.cargo\bin"
if ($env:Path -notlike "*$CargoBinPath*") {
    Write-Host "Adding $CargoBinPath to current session PATH..." -ForegroundColor Yellow
    $env:Path += ";$CargoBinPath"
}

# 2. Check if Cargo/Rust is installed
$cargoInstalled = $false
try {
    $cargoVersion = & cargo --version 2>$null
    if ($cargoVersion) {
        Write-Host "Cargo found: $cargoVersion" -ForegroundColor Green
        $cargoInstalled = $true
    }
} catch {
    # Ignored
}

if (-not $cargoInstalled) {
    Write-Host "Cargo was not found in the PATH." -ForegroundColor Yellow
    
    # Check if rustup-init is present in the cargo bin folder
    $rustupInitPath = Join-Path $CargoBinPath "rustup-init.exe"
    if (Test-Path $rustupInitPath) {
        Write-Host "Found rustup-init at $rustupInitPath. Running installation..." -ForegroundColor Yellow
        & $rustupInitPath -y
        # Reload path after installation
        $env:Path += ";$CargoBinPath"
    } else {
        Write-Host "Rust toolchain is not fully installed." -ForegroundColor Red
        Write-Host "Please download and run the Rust installer: https://rustup.rs/" -ForegroundColor Cyan
        Write-Host "Or run 'winget install Rustup' in a fresh administrative shell, then rerun this script." -ForegroundColor Cyan
        Read-Host "Press Enter to exit..."
        exit
    }
}

# 3. Check if Rust toolchain needs initialization
try {
    $rustcCheck = & rustc --version 2>$null
    if (-not $rustcCheck) {
        Write-Host "Rust toolchain is installed but no default toolchain is active." -ForegroundColor Yellow
        Write-Host "Activating stable toolchain..." -ForegroundColor Yellow
        & rustup default stable
    }
} catch {
    # Ignored
}

# 4. Check if PostgreSQL database is running (Port 5432)
Write-Host "Checking if PostgreSQL is running on port 5432..." -ForegroundColor Cyan
$dbConnection = Test-NetConnection -ComputerName localhost -Port 5432 -WarningAction SilentlyContinue

if ($dbConnection.TcpTestSucceeded) {
    Write-Host "PostgreSQL is running and accessible on port 5432." -ForegroundColor Green
} else {
    Write-Host "PostgreSQL is NOT running on port 5432." -ForegroundColor Yellow
    Write-Host "Attempting to start PostgreSQL via Docker Compose..." -ForegroundColor Yellow
    
    # Try starting docker compose postgres
    try {
        & docker compose up -d postgres
        Write-Host "Waiting for database to initialize (10s)..." -ForegroundColor Yellow
        Start-Sleep -Seconds 10
        
        # Re-check port 5432
        $dbConnection = Test-NetConnection -ComputerName localhost -Port 5432 -WarningAction SilentlyContinue
        if ($dbConnection.TcpTestSucceeded) {
            Write-Host "PostgreSQL container started successfully." -ForegroundColor Green
        } else {
            Write-Host "Could not connect to PostgreSQL even after starting Docker. Please ensure Docker Desktop is running and WSL2 is enabled." -ForegroundColor Red
            Write-Host "If Docker is not working, please start your local PostgreSQL service." -ForegroundColor Red
        }
    } catch {
        Write-Host "Failed to start Docker Compose. Please ensure Docker is installed and running." -ForegroundColor Red
    }
}

# 5. Run the project
Write-Host "Starting the Rust IAM Platform..." -ForegroundColor Green
& cargo run
