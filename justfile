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

# Bump all versions (Cargo.toml, bindings/python, bindings/csharp)
bump version:
    @echo "Bumping version to {{version}}..."
    $c = Get-Content Cargo.toml; $c[2] = 'version = "{{version}}"'; $c | Set-Content Cargo.toml
    $c = Get-Content bindings/python/Cargo.toml; $c[2] = 'version = "{{version}}"'; $c | Set-Content bindings/python/Cargo.toml
    $c = Get-Content bindings/python/pyproject.toml; $c[6] = 'version = "{{version}}"'; $c | Set-Content bindings/python/pyproject.toml
    (Get-Content bindings/csharp/MinaCalc.csproj -Raw) -replace '<Version>\d+\.\d+\.\d+</Version>', '<Version>{{version}}</Version>' | Set-Content bindings/csharp/MinaCalc.csproj
    @echo "Version bumped to {{version}}"

# Release workflow: bump, check, commit, push, tag
release version:
    @echo "Starting release v{{version}}..."
    just bump {{version}}
    just check
    git add -A
    git commit -m "chore: release v{{version}}"
    git push
    git tag -a "v{{version}}" -m "Release v{{version}}"
    git push origin "v{{version}}"
    @echo "Released v{{version}}!"

# Quick QA check only (no commit)
qa:
    just check
