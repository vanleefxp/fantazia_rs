# Format the workspace, then lint every member with warnings denied.
lint:
    cargo fmt --all
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Test every workspace member across all targets and features.
test:
    cargo test --workspace --all-targets --all-features
