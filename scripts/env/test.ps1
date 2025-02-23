# Define container name and database credentials
$ContainerName = "pgvectors-test"
$DB_USER = "loco"
$DB_PASSWORD = "loco"
$DB_NAME = "photos-backend_test"
$Image = "pgvector/pgvector:pg17"

# Check if the container is already running
$ExistingContainer = podman ps -a --format "{{.Names}}" | Where-Object { $_ -eq $ContainerName }

if ($ExistingContainer) {
  Write-Host "Removing existing container: $ContainerName"
  podman stop $ContainerName
  podman rm $ContainerName
}

# Start a fresh PostgreSQL container
Write-Host "Starting new test database container..."
podman run --rm `
  --name $ContainerName `
  -e POSTGRES_USER=$DB_USER `
  -e POSTGRES_PASSWORD=$DB_PASSWORD `
  -e POSTGRES_DB=$DB_NAME `
  -p 5432:5432 `
  -d $Image

Write-Host "Database container has been reset and is running."
