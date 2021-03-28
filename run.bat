@echo off
cargo build
if not ERRORLEVEL 0 goto end
set RUST_LOG=info
target\debug\hex_dump.exe %*
:end