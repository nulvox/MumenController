#!/bin/bash
# Test script for building and verifying different pinout configurations

echo "Building standard pinout configuration (default)..."
cargo build --release
cargo objcopy --release -- -O ihex mumen_standard.hex

echo "Building alternate pinout configuration..."
cargo build --release --features "alternate_pinout"
cargo objcopy --release --features "alternate_pinout" -- -O ihex mumen_alternate.hex

echo "Build process completed."
echo ""
echo "Generated hex files:"
ls -lh *.hex

echo ""
echo "To flash the standard configuration:"
echo "teensy_loader_cli --mcu=IMXRT1062 -w mumen_standard.hex"
echo ""
echo "To flash the alternate configuration:"
echo "teensy_loader_cli --mcu=IMXRT1062 -w mumen_alternate.hex"