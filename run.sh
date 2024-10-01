#!/bin/sh
cargo build --target x86_64-pc-windows-gnu &&
cp target/x86_64-pc-windows-gnu/debug/puzzle-cube.exe . &&
exec ./puzzle-cube.exe "$@"