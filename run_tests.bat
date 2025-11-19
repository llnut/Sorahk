@echo off
REM Test runner for Sorahk on Windows
REM This script runs all tests and provides a summary

echo ============================================
echo   Sorahk Test Suite
echo ============================================
echo.

echo [1/4] Running library unit tests...
cargo test --lib
if %ERRORLEVEL% NEQ 0 (
    echo Library tests failed!
    exit /b 1
)
echo.

echo [2/4] Running integration tests...
cargo test --test integration_tests
if %ERRORLEVEL% NEQ 0 (
    echo Integration tests failed!
    exit /b 1
)
echo.

echo [3/4] Running all tests with verbose output...
cargo test -- --nocapture
if %ERRORLEVEL% NEQ 0 (
    echo Verbose tests failed!
    exit /b 1
)
echo.

echo [4/4] Running tests in release mode...
cargo test --release
if %ERRORLEVEL% NEQ 0 (
    echo Release mode tests failed!
    exit /b 1
)
echo.

echo ============================================
echo   All tests passed successfully!
echo ============================================

