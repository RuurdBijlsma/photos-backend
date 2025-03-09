Write-Host "Check formatting..."
cargo fmt --all
cargo fmt --all -- --check

Write-Host "Running clippy..."
cargo clippy --all-features -- `
    -D warnings -W clippy::pedantic `
    -W clippy::nursery -W rust-2018-idioms `
    -A clippy::single-match-else

# Run Rust tests
Write-Host "Running tests..."
cargo test --all-features --all