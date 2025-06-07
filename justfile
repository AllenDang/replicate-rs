# Justfile for replicate-client

# Default recipe to list available commands
default:
    @just --list

# Run tests
test:
    cargo test

# Run tests with all features
test-all:
    cargo test --all-features

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Check the package
check:
    cargo check --all-targets --all-features

# Build the project
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Run all pre-release checks
pre-release: fmt-check clippy test-all check
    @echo "All pre-release checks passed! ✅"

# Release to crates.io
release: pre-release
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Get current version from Cargo.toml
    current_version=$(cargo pkgid | cut -d# -f2)
    
    echo "🚀 Releasing replicate-client v${current_version} to crates.io..."
    
    # Ensure we're on a clean git state
    if [ -n "$(git status --porcelain)" ]; then
        echo "❌ Git working directory is not clean. Please commit or stash changes."
        exit 1
    fi
    
    # Ensure we're on main/master branch
    current_branch=$(git branch --show-current)
    if [ "$current_branch" != "main" ] && [ "$current_branch" != "master" ]; then
        echo "❌ Not on main/master branch. Currently on: $current_branch"
        exit 1
    fi
    
    # Create and push git tag
    git tag -a "v${current_version}" -m "Release v${current_version}"
    git push origin "v${current_version}"
    
    # Publish to crates.io
    cargo publish
    
    echo "✅ Successfully released replicate-client v${current_version} to crates.io!"
    echo "📦 View at: https://crates.io/crates/replicate-client"

# Dry run release (check what would be published)
release-dry-run: pre-release
    cargo publish --dry-run

# Update dependencies
update:
    cargo update

# Generate documentation
docs:
    cargo doc --all-features --no-deps --open

# Run security audit
audit:
    cargo audit 