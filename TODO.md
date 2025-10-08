* ✅ copy setup-related endpoints from old backend
* ✅ set up new api backend.
* Show photos
* clean refresh token table on schedule
* ✅ fix shitty refresh token finding
* rate limit met tower-http::limit voor /login en /auth/refresh en password reset endpoint als ik die krijg
* cors met tower-http::cors
* password reset flow (email) (make mail optional)
* look at rust config package
* add expiry time to auth responses
* ✅ fix errors in api, abstraction for it, probably.
* auth integration test:
  1. clear db
  2. http://localhost:3567/auth/register
  3. http://localhost:3567/auth/login
  4. use access_token on http://localhost:3567/auth/me -> verify
  5. set user role to USER
  6. http://localhost:3567/auth/admin-check -> should be forbidden
  7. set user role to ADMIN
  8. http://localhost:3567/auth/admin-check -> should work
  9. http://localhost:3567/auth/refresh -> should work, store refresh_token output
  10. re-run with old refresh_token -> should not work, token is rotated
  11. re-run with stored refresh_token -> should work, store access_token
  12. try access token on get_me
  13. http://localhost:3567/auth/logout
  14. try http://localhost:3567/auth/refresh -> should not work
* ✅ Api docs swagger
* use db config when setting up db. (pool size etc.)
* i accidentally made this a new repo, original was photos-processing
* ✅ in auth/model, split db models and api interfaces