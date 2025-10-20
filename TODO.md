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
    * refrehs materialized view of amount of photos per month
* Show photos in ui:
  * ✅ make endpoint: get photos by month, ui handles which month to fetch
  * ✅ make endpoint: get timeline summary -> get list of every month with amount of photos for that month. (per user)
  * ✅ moet nog een photo density endpoint hebben om de scrollbar density te laten zien.
  * ✅ nieuwe dag is niet altijd newline in de photos grid, misschien toch weer over gaan naar maanden requesten. verder is geminis photos endpoints wel ok.
  * make postgres materialized view for amount of photos per month. Refresh on schedule (maybe start without materiazlied view, if the view is fast enough)
  * ✅ data_url veld in db is useless denk ik (ook in alle analyzers)
  * virtual scroll waar elke maand 1 virtual scroll item is? of elke row is 1 virtual item??
* api:
    * Show photos in ui
    * change the json output of vec<photo> to have small field names (is like 50% smaller)
      * {i: "3d_8yhfd9", "w":1200, "h":1000, "t": "2018-08-30", "v": false}
      * {id: "3d_8yhfd9", "width":1200, "height":1000, "taken_at_local": "2018-08-30", "is_video": false}
    * rate limit met tower-http::limit voor /login en /auth/refresh en password reset endpoint als ik die krijg
    * ✅ cors met tower-http::cors
    * password reset flow (email) (make mail optional)
    * add expiry time to auth responses (zit er al in via jwt, moet dat nog? ik denk t wel)
    * only allow register if no user exists, or if a valid invite token is passed
    * frontend tip: maybe put each row in a lazyload? or skeleton loader, or stop loading='lazy' op img tags
    * ✅ add random image + theme endpoint
* integration test
* check of readme uitleg klopt met verse windows installatie & linux
* update sqlx
* When we delete user, make sure to delete the jobs of that user (maak job type delete user)
* user_id from relative path is broken (it looks for username in first path of path, but we use media folder now in the
  db, so we'll somehow have to get this. I think only way is loop over all media_folders in db and see if file path
  starts_with each media_folder)
* Improve last_error field in jobs, just put entire report in there?
* a lotta failed jobs
* add time_utc to media_item table, use it for sorting with COALESCE.
* ✅ rename taken_at_local to taken_at_local
* camelCase elke interfaces.rs struct

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
