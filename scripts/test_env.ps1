podman run --rm `
  --name pgvectors-test `
  -e POSTGRES_USER=loco `
  -e POSTGRES_PASSWORD=loco `
  -e POSTGRES_DB=photos-backend_test `
  -p 5432:5432 `
  -v pgdata-test:/var/lib/postgresql/data `
  -d pgvector/pgvector:pg17