# make sure sqlx-cli is installed:
# cargo install sqlx-cli --no-default-features --features postgres

Push-Location (Join-Path $PSScriptRoot "..")

./scripts/disconnect_postgres.ps1

# Drop database and recreate from migration
sqlx database drop -y
sqlx database create
sqlx migrate run

Pop-Location
