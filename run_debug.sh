#!/bin/bash
export XDG_DATA_DIRS="$HOME/.local/share:/opt/homebrew/share:/usr/local/share:/usr/share"
export GSETTINGS_SCHEMA_DIR="/opt/homebrew/share/glib-2.0/schemas"
export RUST_LOG=info
exec ./target/debug/rustconn
