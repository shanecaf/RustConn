#!/bin/bash
cargo build -p rustconn --no-default-features \
  --features "vnc-embedded,rdp-embedded,rdp-audio" 2>&1 | \
  grep -E "error|warning.*unused" | head -20
echo "Exit: $?"
