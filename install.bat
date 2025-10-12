@echo off
setlocal enabledelayedexpansion
REM Synapse Installation Script for Windows
REM Compiles and installs Synapse to %USERPROFILE%\.synapse\bin with unified data directory

echo 🚀 Synapse Installation Script for Windows
echo ==============================

REM Prevent window from closing on errors
if not defined IN_SUBPROCESS (
    set IN_SUBPROCESS=1
    "%~f0" %*
    echo.
    echo Script completed. Press any key to exit...
    pause >nul
    exit /b
)

REM Check if we're in the correct directory
if not exist "Cargo.toml" (
    echo ❌ Error: This script must be run from the Synapse project directory
    echo    Make sure you're in the directory containing the workspace Cargo.toml
    exit /b 1
)

findstr /C:"members = [\"synapse-core\"" Cargo.toml >nul
if %errorlevel% neq 0 (
    echo ❌ Error: This script must be run from the Synapse project directory
    echo    Make sure you're in the directory containing the workspace Cargo.toml
    exit /b 1
)

REM Check if Rust/Cargo is installed
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Cargo not found. Installing Rust...
    echo 📥 Downloading and installing Rust via rustup-init.exe
    
    REM Check if PowerShell is available for download
    powershell -Command "Get-Command powershell" >nul 2>&1
    if %errorlevel% equ 0 (
        echo Downloading Rust installer...
        powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile 'rustup-init.exe'"
        if exist "rustup-init.exe" (
            echo Running Rust installer...
            rustup-init.exe -y --default-toolchain stable
            del rustup-init.exe
            
            REM Refresh PATH to include Cargo
            set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
            
            REM Verify installation
            cargo --version >nul 2>&1
            if %errorlevel% equ 0 (
                echo ✅ Rust installed successfully!
                for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
                echo 📍 Location: %USERPROFILE%\.cargo\bin\cargo.exe
            ) else (
                echo ❌ Error: Rust installation failed
                echo    Please install manually: https://rustup.rs/
                exit /b 1
            )
        ) else (
            echo ❌ Error: Failed to download Rust installer
            echo    Please install manually: https://rustup.rs/
            exit /b 1
        )
    ) else (
        echo ❌ Error: PowerShell not available. Cannot install Rust automatically
        echo    Please install Rust manually: https://rustup.rs/
        exit /b 1
    )
) else (
    for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
    echo ✅ Found Cargo: %CARGO_VERSION%
)

REM Create Synapse data directory
echo 📁 Creating Synapse data directory...
if not exist "%USERPROFILE%\.synapse" mkdir "%USERPROFILE%\.synapse"
if not exist "%USERPROFILE%\.synapse\data" mkdir "%USERPROFILE%\.synapse\data"
if not exist "%USERPROFILE%\.synapse\logs" mkdir "%USERPROFILE%\.synapse\logs"
if not exist "%USERPROFILE%\.synapse\config" mkdir "%USERPROFILE%\.synapse\config"

REM Create bin directory
echo 📁 Creating Synapse bin directory...
if not exist "%USERPROFILE%\.synapse\bin" mkdir "%USERPROFILE%\.synapse\bin"

REM Kill any running synapse processes
echo 🔄 Stopping any running Synapse processes...
taskkill /f /im synapse.exe >nul 2>&1
timeout /t 1 /nobreak >nul

REM Build release version
echo 🔨 Building Synapse (release mode)...
cargo build --release --package synapse-cli
if %errorlevel% neq 0 (
    echo ❌ Error: Build failed
    exit /b 1
)

REM Build frontend
echo.
echo ========================================
echo 🎨 Building frontend...
echo ========================================
echo.

REM Check if npm is available
echo Checking for npm...
where npm
set NPM_CHECK=!errorlevel!
echo npm check returned: !NPM_CHECK!

if !NPM_CHECK! neq 0 (
    echo.
    echo ❌ npm not found! Node.js is required to build the frontend.
    echo.
    echo Checking if npm.cmd exists...
    where npm.cmd
    echo.
    echo Current PATH:
    echo !PATH!
    echo.
    echo Please install Node.js from https://nodejs.org/
    echo Then run this installer again.
    echo.
    echo ⚠️  Continuing without frontend ^(dashboard will not work^)
    echo.
    echo Press any key to continue without frontend...
    pause >nul
    goto skip_frontend_build
)

echo ✅ npm found!
call npm --version
echo.

REM Check if wasm-pack is installed for WASM build
echo Checking for wasm-pack...
wasm-pack --version >nul 2>&1
if !errorlevel! neq 0 (
    echo ⚠️  wasm-pack not found - installing it now...
    echo This is required to build the WASM performance module
    cargo install wasm-pack
    if !errorlevel! neq 0 (
        echo ❌ Error: Failed to install wasm-pack
        echo Please install it manually: cargo install wasm-pack
        exit /b 1
    )
    echo ✅ wasm-pack installed successfully!
) else (
    for /f "tokens=*" %%i in ('wasm-pack --version') do set WASM_PACK_VERSION=%%i
    echo ✅ Found wasm-pack: !WASM_PACK_VERSION!
)
echo.

echo 📁 Changing to frontend directory...
cd synapse-web\frontend-react
if %errorlevel% neq 0 (
    echo ❌ Error: Could not change to synapse-web\frontend-react directory
    pause
    exit /b 1
)
echo Current directory: %CD%
echo.

echo 🧹 Cleaning previous node_modules and package-lock.json...
if exist "node_modules" rmdir /s /q node_modules
if exist "package-lock.json" del /f /q package-lock.json
echo.

echo 📦 Running npm install...
echo [This may take a few minutes, please wait...]
call npm install > npm-install.log 2>&1
set NPM_INSTALL_ERROR=!errorlevel!
echo npm install returned exit code: !NPM_INSTALL_ERROR!

if !NPM_INSTALL_ERROR! neq 0 (
    echo ❌ Error: npm install failed with exit code !NPM_INSTALL_ERROR!
    echo.
    echo Last 20 lines of npm output:
    type npm-install.log | more +1
    echo.
    cd ..\..
    del npm-install.log 2>nul
    echo Script failed. Window will stay open.
    exit /b 1
)
echo ✅ npm install completed successfully
del npm-install.log 2>nul
echo.

echo 🔨 Running npm run build (includes WASM)...
echo [This may take a few minutes, please wait...]
call npm run build > npm-build.log 2>&1
set NPM_BUILD_ERROR=!errorlevel!
echo npm run build returned exit code: !NPM_BUILD_ERROR!

if !NPM_BUILD_ERROR! neq 0 (
    echo ❌ Error: Frontend build failed with exit code !NPM_BUILD_ERROR!
    echo.
    echo Last 20 lines of npm output:
    type npm-build.log | more +1
    echo.
    cd ..\..
    del npm-build.log 2>nul
    echo Script failed. Window will stay open.
    exit /b 1
)
echo ✅ Frontend build completed successfully
del npm-build.log 2>nul
echo.

echo 📁 Returning to workspace root...
cd ..\..
echo Current directory: %CD%
echo.

REM Create frontend directory in release target
echo 📦 Preparing frontend files for installation...
if not exist "target\release\frontend" mkdir "target\release\frontend"

echo 📋 Checking if dist directory exists...
if not exist "synapse-web\frontend-react\dist" (
    echo ❌ Error: dist directory not found at synapse-web\frontend-react\dist
    echo The build may have failed silently
    pause
    exit /b 1
)

if not exist "synapse-web\frontend-react\dist\index.html" (
    echo ❌ Error: index.html not found in dist directory
    echo Build completed but output is invalid
    pause
    exit /b 1
)

echo ✅ Frontend build output verified
echo.

echo 📂 Copying frontend files to target\release\frontend...
xcopy /E /I /Y "synapse-web\frontend-react\dist\*" "target\release\frontend\"
set XCOPY_ERROR=%errorlevel%
if %XCOPY_ERROR% neq 0 (
    echo ❌ Error: xcopy failed with exit code %XCOPY_ERROR%
    pause
    exit /b 1
)

if not exist "target\release\frontend\index.html" (
    echo ❌ Error: Frontend files not copied correctly - index.html not found in target
    pause
    exit /b 1
)
echo ✅ Frontend files prepared successfully
echo.

:skip_frontend_build

if not exist "target\release\synapse.exe" (
    echo ❌ Error: Build failed - executable not found
    exit /b 1
)

REM Get file size
for %%A in ("target\release\synapse.exe") do set SIZE=%%~zA
echo ✅ Build successful! Executable size: %SIZE% bytes

REM Install to bin directory
echo 📦 Installing to %USERPROFILE%\.synapse\bin\synapse.exe...
copy "target\release\synapse.exe" "%USERPROFILE%\.synapse\bin\"
if %errorlevel% neq 0 (
    echo ❌ Error: Failed to copy executable
    exit /b 1
)

REM Install frontend files
echo 🎨 Installing frontend files...
if exist "target\release\frontend\index.html" (
    if not exist "%USERPROFILE%\.synapse\bin\frontend" mkdir "%USERPROFILE%\.synapse\bin\frontend"

    REM Copy all frontend files recursively with proper structure
    xcopy /E /I /Y /Q "target\release\frontend" "%USERPROFILE%\.synapse\bin\frontend"

    REM Verify installation
    if exist "%USERPROFILE%\.synapse\bin\frontend\index.html" (
        echo ✅ Frontend files installed successfully
        echo 📁 Frontend location: %USERPROFILE%\.synapse\bin\frontend
        dir /B "%USERPROFILE%\.synapse\bin\frontend" | findstr /R ".*" > nul
        if %errorlevel% equ 0 (
            echo 📄 Frontend files count:
            dir /B "%USERPROFILE%\.synapse\bin\frontend" | find /C /V ""
        )
    ) else (
        echo ❌ Error: Frontend installation failed - index.html not copied
        echo 🔍 Debug: Checking target\release\frontend contents...
        dir /B "target\release\frontend"
        exit /b 1
    )
) else (
    echo ⚠️  Warning: Frontend files not found in target\release\frontend\index.html
    echo    The web interface may not work correctly
    if exist "target\release\frontend" (
        echo 🔍 Debug: Contents of target\release\frontend:
        dir /B "target\release\frontend"
    )
)

REM Set up environment configuration
echo ⚙️ Setting up environment configuration...

REM Escape backslashes for TOML configuration
set ESCAPED_USERPROFILE=%USERPROFILE:\=\\%

echo # Synapse Configuration > "%USERPROFILE%\.synapse\config\config.toml"
echo # Generated by install.bat >> "%USERPROFILE%\.synapse\config\config.toml"
echo. >> "%USERPROFILE%\.synapse\config\config.toml"
echo data_dir = "%ESCAPED_USERPROFILE%\\.synapse\\data" >> "%USERPROFILE%\.synapse\config\config.toml"
echo log_level = "info" >> "%USERPROFILE%\.synapse\config\config.toml"
echo. >> "%USERPROFILE%\.synapse\config\config.toml"
echo [ai] >> "%USERPROFILE%\.synapse\config\config.toml"
echo default_provider = "openrouter" >> "%USERPROFILE%\.synapse\config\config.toml"
echo. >> "%USERPROFILE%\.synapse\config\config.toml"
echo [database] >> "%USERPROFILE%\.synapse\config\config.toml"
echo path = "%ESCAPED_USERPROFILE%\\.synapse\\data\\synapse.db" >> "%USERPROFILE%\.synapse\config\config.toml"
echo. >> "%USERPROFILE%\.synapse\config\config.toml"
echo [dashboard] >> "%USERPROFILE%\.synapse\config\config.toml"
echo port = 3000 >> "%USERPROFILE%\.synapse\config\config.toml"
echo host = "127.0.0.1" >> "%USERPROFILE%\.synapse\config\config.toml"
echo. >> "%USERPROFILE%\.synapse\config\config.toml"
echo [mcp_server] >> "%USERPROFILE%\.synapse\config\config.toml"
echo port = 3001 >> "%USERPROFILE%\.synapse\config\config.toml"
echo host = "127.0.0.1" >> "%USERPROFILE%\.synapse\config\config.toml"

REM Create Windows service script (optional)
echo 🔧 Creating Windows service script (optional)...
echo @echo off > "%USERPROFILE%\.synapse\config\start-synapse-mcp.bat"
echo REM Start Synapse MCP Server as a background process >> "%USERPROFILE%\.synapse\config\start-synapse-mcp.bat"
echo start /B "" "%USERPROFILE%\.synapse\bin\synapse.exe" --mcp-server >> "%USERPROFILE%\.synapse\config\start-synapse-mcp.bat"
echo echo Synapse MCP Server started in background >> "%USERPROFILE%\.synapse\config\start-synapse-mcp.bat"
echo echo To stop it, run: taskkill /f /im synapse.exe >> "%USERPROFILE%\.synapse\config\start-synapse-mcp.bat"

echo 💡 To start MCP server as background service, run:
echo    %USERPROFILE%\.synapse\config\start-synapse-mcp.bat
echo    To stop it: taskkill /f /im synapse.exe

REM Check if Synapse bin directory is in PATH
echo %PATH% | findstr /C:"%USERPROFILE%\.synapse\bin" >nul
if %errorlevel% neq 0 (
    echo ⚠️  Warning: %USERPROFILE%\.synapse\bin is not in your PATH
    echo    Add it using:
    echo    set PATH="%%USERPROFILE%%\.synapse\bin;%%PATH%%"
    echo    Or add it permanently through System Properties ^> Environment Variables
    echo.
)

REM Check if Cargo bin directory is in PATH (for newly installed Rust)
echo %PATH% | findstr /C:"%USERPROFILE%\.cargo\bin" >nul
if %errorlevel% neq 0 (
    echo ⚠️  Warning: %USERPROFILE%\.cargo\bin is not in your PATH
    echo    Add it using:
    echo    set PATH="%%USERPROFILE%%\.cargo\bin;%%PATH%%"
    echo    Or add it permanently through System Properties ^> Environment Variables
    echo.
)

REM Test installation
echo 🧪 Testing installation...
"%USERPROFILE%\.synapse\bin\synapse.exe" --version >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ Synapse installed successfully!
    echo 📍 Location: %USERPROFILE%\.synapse\bin\synapse.exe
    echo.
    echo Usage examples:
    echo   synapse --help                    # Show help
    echo   synapse --file C:\logs\app.log   # Analyze log file
    echo   synapse --dashboard               # Start web dashboard
    echo   synapse --mcp-server              # Start MCP server (stdio mode)
    echo   synapse --mcp-server --mcp-transport http  # Start MCP server (HTTP mode)
    echo   synapse --mcp-server --mcp-port 8080       # Start MCP server on custom port
    echo   synapse init                      # Initialize project
    echo.
    echo Data directory: %USERPROFILE%\.synapse\data
    echo Configuration: %USERPROFILE%\.synapse\config\config.toml
    echo.
    echo MCP Server tools available:
    echo   - list_projects: List available Synapse projects
    echo   - get_project: Get detailed project information
    echo   - list_analyses: List analyses for a project
    echo   - get_analysis: Get complete analysis results
    echo   - get_analysis_status: Get analysis status for polling
    echo   - analyze_file: Trigger new analysis on existing file
    echo.
    echo Docker usage:
    echo   docker run -p 8080:8080 -v %USERPROFILE%\.synapse\data:/app/data synapse --dashboard
    echo   docker run -p 3001:3001 -v %USERPROFILE%\.synapse\data:/app/data synapse --mcp-server
) else (
    echo ❌ Installation failed - synapse.exe not working
    exit /b 1
)

echo 🎉 Installation complete!
echo.
echo To use Synapse from anywhere, add %USERPROFILE%\.synapse\bin to your PATH
echo or run it directly from: %USERPROFILE%\.synapse\bin\synapse.exe
echo.
echo Press any key to close this window...
pause >nul