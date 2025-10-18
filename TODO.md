* ✅ copy setup-related endpoints from old backend
* ✅ set up new api backend.
* ✅ fix shitty refresh token finding
* ✅ fix errors in api, abstraction for it, probably.
* ✅ Api docs swagger
* ✅ in auth/model, split db models and api interfaces
* ✅ users have to be implemented in photos processing at some point (media item must have user id) (user folders)
* ✅ I accidentally made this a new repo, original was photos-processing
* ✅ use db config when setting up db. (pool size etc.)
* ✅ als een crate de settings retrieved voordat dotenv geladen is gaat het stuk.
* ✅ look at rust config package
* ✅ avif not supported by visual analyzer
* ✅ Add some kind of cli flag to specify that a worker can't work on ML type of job
* ✅ BUG als een worker dood gaat terwijl een job aan het running is dan blijft ie running en pakt niemand m meer op.
* ✅ ML Analysis:
    * ✅ Make ML jobtype, give priority below videos (30?) so they are done last
    * ✅ color data from python, make in rust
    * ✅ captioner logic in rust (all the questions like is_animal)
    * ✅ quality measure from python, make in rust
    * ✅ make required sql migration tables for ML analysis
    * ✅ handle machine learning analysis job, put in db
* schedule runner -> might have to use ofelia or kubernetes+helm to get clean cronjobs.
    * ✅ indexing
    * ✅ clean refresh token table on schedule
    * clustering on schedule
* api:
    * Show photos in ui
    * rate limit met tower-http::limit voor /login en /auth/refresh en password reset endpoint als ik die krijg
    * cors met tower-http::cors
    * password reset flow (email) (make mail optional)
    * add expiry time to auth responses (zit er al in via jwt, moet dat nog? ik denk t wel)
    * only allow register if no user exists, or if a valid invite token is passed
    * add random image + theme endpoint
* integration test
* check of readme uitleg klopt met verse windows installatie & linux
* update sqlx
* When we delete user, make sure to delete the jobs of that user (maak job type delete user)
* user_id from relative path is broken (it looks for username in first path of path, but we use media folder now in the
  db, so we'll somehow have to get this. I think only way is loop over all media_folders in db and see if file path
  starts_with each media_folder)

## Kubernetes vs Docker compose

+ Met coole UI kan je dingen inzien
+ Cronjobs geintegreerd
+ Voelt professioneel
+ als chatgpt te geloven is, makkelijke setup (installs k3s -> edit values.yaml -> run)
+ service voor frontend, is ervoor gemaakt
+ kan op een hosting service makkelijker

- Schrikt selfhosters af
- complexe templates & charts
- gebruikt meer resources dan docker compose
- meer omslachtige mounting van schijven
- meer complicated troubleshooting, logs enzo
