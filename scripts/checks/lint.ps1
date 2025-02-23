Write-Host "Check formatting..."
cargo fmt --all
cargo fmt --all -- --check

Write-Host "Running clippy..."
cargo clippy --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W rust-2018-idioms