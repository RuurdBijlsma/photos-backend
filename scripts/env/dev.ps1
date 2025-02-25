podman run --rm `
  --name pgvectors `
  -e POSTGRES_USER=loco `
  -e POSTGRES_PASSWORD=loco `
  -e POSTGRES_DB=photos-backend_development `
  -p 5432:5432 `
  -v pgdata:/var/lib/postgresql/data `
  -d pgvector/pgvector:pg17

#podman run --rm `
#  --name photos-redis `
#  -p 6379:6379 `
#  -d redis
