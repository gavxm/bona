# Run the full CI check suite locally.
check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo deny check
    cargo test

# Format code.
fmt:
    cargo fmt

# Run tests.
test:
    cargo test

# Run tests, re-running on file changes.
watch:
    cargo watch -x test

# Audit dependencies for license and vulnerability issues.
deny:
    cargo deny check

# Investigate a model (ex. `just run meta-llama/Llama-3.1-8B-Instruct`).
run model_id:
    cargo run -- investigate {{model_id}}

# Investigate a model and output JSON.
run-json model_id:
    cargo run -- investigate {{model_id}} --json

# Review pending insta snapshots.
snapshots:
    cargo insta review
