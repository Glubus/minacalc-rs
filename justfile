# MinaCalc-rs Justfile
# QA workflow: test, fmt, commit, push, tag

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

# QA workflow: check, commit, push
qa:
    just check
