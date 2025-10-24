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
* ✅ add time_utc to media_item table
* ✅ rename taken_at_local to taken_at_local
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
    * ? refrehs materialized view of amount of photos per month
* Show photos in ui:
    * ✅ make endpoint: get photos by month, ui handles which month to fetch
    * ✅ make endpoint: get timeline summary -> get list of every month with amount of photos for that month. (per user)
    * ✅ moet nog een photo density endpoint hebben om de scrollbar density te laten zien.
    * ✅ nieuwe dag is niet altijd newline in de photos grid, misschien toch weer over gaan naar maanden requesten.
    * ✅ data_url veld in db is useless denk ik (ook in alle analyzers)
    * ? make postgres materialized view for amount of photos per month. Refresh on schedule (maybe start without
      materiazlied view, if the view is fast enough)
    * virtual scroll waar elke maand 1 virtual scroll item is? of elke row is 1 virtual item??
* api:
    * ✅ add random image + theme endpoint
    * ✅ cors met tower-http::cors
    * ✅ change the json output of vec<photo> to have small field names (is like 50% smaller)
    * Show photos in ui
    * rate limit met tower-http::limit voor /login en /auth/refresh en password reset endpoint als ik die krijg
    * password reset flow (email) (make mail optional)
    * add expiry time to auth responses (zit er al in via jwt, moet dat nog? ik denk t wel)
    * ✅ only allow register if no user exists
    * Make invite token functionality for registering new user. (Admin sets the folder, linked to the invite token in
      db, when invite token is used and user is created, delete invite token row and put media folder linked to the new
      user account)
    * frontend tip: maybe put each row in a lazyload? or skeleton loader, or stop loading='lazy' op img tags
* integration test
    * auth
    * "setup"
    * ingest
    * retrieve
* check of readme uitleg klopt met verse windows installatie & linux
* update sqlx
* When we delete user, make sure to delete the jobs of that user (maak job type delete user)
* !BUG user_id from relative path is broken (it looks for username in first path of path, but we use media folder now in
  the
  db, so we'll somehow have to get this. I think only way is loop over all media_folders in db and see if file path
  starts_with each media_folder)
* Improve last_error field in jobs, just put entire report in there?
* a lotta failed jobs
* use time_utc for sorting with COALESCE (don't use it for binning into months and such, and don't return the utc time
  to user)
* camelCase elke interfaces.rs struct
* monitoring/alerting
    * prometheus
    * grafana
    * alertmanager
    * loki? denk t niet
* protobuf for more endpoints?
* i made the photos handler/service code garbage. clean up pls.
* Dont use single character field names now that we use protobuf for big requests
* look into not using generated code, just add the prost annotations on the real structs
* use proper index on get-month endpoint, if not already at max perf level.

## Kubernetes vs Docker compose (of beide? in eigen repos?)

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
