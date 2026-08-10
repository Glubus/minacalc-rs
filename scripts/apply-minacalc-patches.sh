#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

# build.rs copies pristine c_code into target/, applies the patch there,
# compiles it, and removes the temporary source tree automatically.
cargo build --manifest-path "$repository_root/Cargo.toml" -p minacalc-sys
echo "Verified the temporary MinaCalc patch build; vendored sources were untouched."
