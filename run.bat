@echo off
cargo build
if ERRORLEVEL 1 goto end
set RUST_LOG=info
target\debug\hex_dump.exe %*
:end