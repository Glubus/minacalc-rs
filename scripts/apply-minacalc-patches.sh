#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
patch_file="$repository_root/patches/minacalc-configurable-calc-settings.patch"

if git -C "$repository_root" apply --check "$patch_file" 2>/dev/null; then
    git -C "$repository_root" apply "$patch_file"
    echo "Applied configurable calculator settings patch."
elif git -C "$repository_root" apply --reverse --check "$patch_file" 2>/dev/null; then
    echo "Configurable calculator settings patch is already applied."
else
    echo "Unable to apply $patch_file; the vendored MinaCalc sources have changed." >&2
    echo "Resolve the patch against the new upstream version, then run the test suite." >&2
    exit 1
fi
