Write-Host "Check formatting..."
cargo fmt --all -- --check

Write-Host "Running clippy..."
cargo clippy --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W rust-2018-idioms

# Set up the DATABASE_URL environment variable
$env:DATABASE_URL = "postgres://loco:loco@localhost:5432/photos-backend_test"
Write-Host "DATABASE_URL set to: $env:DATABASE_URL"

# Run Rust tests
Write-Host "Running tests..."
cargo test --all-features --all