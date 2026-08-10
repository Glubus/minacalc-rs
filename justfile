# MinaCalc-rs Justfile
# QA and Release workflows

set windows-shell := ["pwsh", "-NoLogo", "-Command"]

# Default recipe
default:
    @just --list

# Run all tests
test:
    cargo test --all-features

# Check formatting
fmt:
    cargo fmt --all -- --check

# Format code (fix)
fmt-fix:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --all-features -- -D warnings

# Full QA check (test + fmt + clippy)
check:
    just test
    just fmt
    just clippy

# Validate locally before following RELEASING.md and pushing a version tag
release-check:
    just check
    @echo "Release checks passed. Follow RELEASING.md to publish."

# Quick QA check only (no commit)
qa:
    just check

# Verify that the temporary build copy still accepts the MinaCalc patch
patch-minacalc:
    ./scripts/apply-minacalc-patches.sh
