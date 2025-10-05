# Use this script to set the env variables from .env, so you can use sqlx-cli commands.
Push-Location (Join-Path $PSScriptRoot "..")

# Read from .env file
get-content .env | foreach {
    $name, $value = $_.split('=')
    set-content env:\$name $value
}

Pop-Location