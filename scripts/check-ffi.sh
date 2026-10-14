#!/bin/sh
# Verifies src/sys/ffi.rs against the system headers (needs gcc + python3).
set -eu
here=$(cd "$(dirname "$0")" && pwd)
tmp=$(mktemp -d)
gcc -O0 -o "$tmp/ffi_check" "$here/ffi_check.c"
"$tmp/ffi_check" > "$tmp/c.txt"
python3 "$here/check_ffi.py" "$tmp/c.txt" "$here/../src/sys/ffi.rs"
