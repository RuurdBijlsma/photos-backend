# make sure sqlx-cli is installed:
# cargo install sqlx-cli --no-default-features --features postgres

Push-Location (Join-Path $PSScriptRoot "..")

./scripts/set_env.ps1

# Drop database and recreate from migration
sqlx database drop -y
sqlx database create

# Running the program will run migrations, so not needed here.

Pop-Location