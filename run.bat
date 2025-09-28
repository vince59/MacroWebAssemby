@echo off
del C:\rust\MacroWebAssemby\exemple\hello.wat
cargo run C:\rust\MacroWebAssemby\exemple\hello.gfr
del C:\rust\MacroWebAssemby\www\binary.wasm
C:\rust\MacroWebAssemby\bin\wat2wasm.exe C:\rust\MacroWebAssemby\exemple\hello.wat -o C:\rust\MacroWebAssemby\www\binary.wasm
