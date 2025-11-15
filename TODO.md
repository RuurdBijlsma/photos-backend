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
* ✅ camelCase elke interfaces.rs struct
* ✅ protobuf for more endpoints?
* ✅ i made the photos handler/service code garbage. clean up pls.
* ✅ Dont use single character field names now that we use protobuf for big requests
* ✅ 👎 look into not using generated code, just add the prost annotations on the real structs
* ✅ response size of by-month.pb is about 51 kb, so why is the request so slow? request on rust end is around 25-30 ms,
  but on frontend end is 100-125 ms.
* ✅ make ratios endpoint more of a timeline endpoint, with count per month.
* ✅ thumbnails zijn gedraait (orientation tag exif)
* ✅ by-month and timeline dont return in sync media items. timeline ratios is wrong, it's not in order of
  taken_at_local.
* ✅ use time_utc for sorting with COALESCE (don't use it for binning into months and such, and don't return the utc time
  to user)
* ✅ Fix failed analysis jobs
* ✅ Refresh auth wordt niet goed gedaan in frontend.
* ✅ !BUG user_id from relative path is broken
* ✅ heb ik met de nieuwe fallback timezone 0 null's in taken at utc? ja maar dat is een leugen dus ik haal t weg
* ✅ refresh token gives 415 for some reason.
* ✅ add llm to py interop
* ✅ Improve last_error field in jobs, just put entire report in there?
* ✅ now that i have sort_timezone in the db, should i still use fallback timezone to calculate time_utc?
* ✅ visual analysis should have frame percentage or something as a column.
* ✅ ML Analysis:
    * ✅ Make ML jobtype, give priority below videos (30?) so they are done last
    * ✅ color data from python, make in rust
    * ✅ captioner logic in rust (all the questions like is_animal)
    * ✅ quality measure from python, make in rust
    * ✅ make required sql migration tables for ML analysis
    * ✅ handle machine learning analysis job, put in db
* ✅ schedule runner -> might have to use ofelia or kubernetes+helm to get clean cronjobs.
    * ✅ indexing
    * ✅ clean refresh token table on schedule
    * ✅ clustering on schedule
* ✅ Show photos in ui:
    * ✅ make endpoint: get photos by month, ui handles which month to fetch
    * ✅ make endpoint: get timeline summary -> get list of every month with amount of photos for that month. (per user)
    * ✅ moet nog een photo density endpoint hebben om de scrollbar density te laten zien.
    * ✅ nieuwe dag is niet altijd newline in de photos grid, misschien toch weer over gaan naar maanden requesten.
    * ✅ data_url veld in db is useless denk ik (ook in alle analyzers)
    * ✅ virtual scroll waar elke maand 1 virtual scroll item is? of elke row is 1 virtual item??
* ✅ pending_album_media_items isnt getting used
* ✅ Change album id from uuid to niceid (no longer univerally unique requirement)
* ✅ [BUG] pending media items seems to be not used again
* ✅ worker does not output logs to stdout anymore.
* ✅ store_media en store_visual_analysis (met de macros) moet in common_services/database
* ✅ make invite check work with "localhost:9475" instead of "http://localhost:9475" and make it work with https. (it
  currently assumes http).
* ✅ improve OCR
* ✅ [BUG] scan enqueues duplicate jobs if the photo isn't processed yet.
* ✅ [BUG] if album name for /albums/invite/accept is already a folder in media_dir/user_folder, then it doesn't work
  properly.
* ✅ rename details to media_details
* ✅ rename setup to onboarding
* ✅ don't allow start onboarding endpoint if onboarding is already done.
* ✅ remove unused crates
* ✅ add remote_user_id as collaborator to album.
* api:
    * ✅ add random image + theme endpoint
    * ✅ cors met tower-http::cors
    * ✅ change the json output of vec<photo> to have small field names (is like 50% smaller)
    * ✅ Show photos in ui
    * ✅ only allow register if no user exists
    * ✅ frontend tip: maybe put each row in a lazyload? or skeleton loader, or stop loading='lazy' op img tags
    * rate limit met tower-http::limit voor /login en /auth/refresh en password reset endpoint als ik die krijg
    * password reset flow (email) (make mail optional)
    * add expiry time to auth responses (zit er al in via jwt, moet dat nog? ik denk t wel)
    * Make invite token functionality for registering new user. (Admin sets the folder, linked to the invite token in
      db, when invite token is used and user is created, delete invite token row and put media folder linked to the new
      user account)
    * axum-gate? crate voor axum auth
* integration test
    * auth
    * onboarding
    * ingest
    * retrieve
    * album
    * cross server album
* check of readme uitleg klopt met verse windows installatie & linux
* update sqlx
* When we delete user, make sure to delete the jobs of that user (maak job type delete user)
* monitoring/alerting
    * prometheus
    * grafana
    * alertmanager
    * loki? denk t niet
* api performance
    * use proper index on get-month endpoint, if not already at max perf level.
    * at some point copy paste all sql queries into gemini en ask for proper indices
    * timeline_summary.sql en ratios_summary.sql migrations deleten, en weer maken met goeie nieuwe columns (maybe its
      already pretty good).
    * Summary table voor ratios
    * performance check voor beide /photos endpoints met 100k photos erin (explain analyze, check of frontend js veel
      delay toevoegt)
* make sure cache control on thumbnails are immutable/max age.
* fun "albums" notifications & in UI frontpage
    * refresh daily (changes daily): "10 years ago today" -> as long as there's enough photos on that day.
    * refresh weekly ofzo? (only changes with significantly more photos): embedding cluster with LLM name ("Swimming at
      the lake", "Cat pics")
    * group by  (only changes with significantly more photos)
        * caption columns ("setting", "main subject", "is_outside & sunset & ...")
        * group by country?
        * group by animal type?
    * make sure each "fun album" is shown as notification only once. In UI it can be more often?
* split routes/photos into timeline related and media item related
* websocket om nieuwe foto events te sturen
* clean up error and warn and info tracing logs
    * error for fatal boys
    * warn for user might be impacted
    * info for info
* repeated code in import album en import album item worker job, repeated code is in api/s2s en api/albums
    * ✅ parse url stuff
    * ✅ parse token maybe?
    * ✅ share reqwest client via application state and worker context so it's not made every time.
    * ✅ Improve structure of common structs in common photos. (job_payloads.rs ofzo erbij?)
    * get s2s invite summary
    * make s2s client in common code somewhere, to call s2s endpoints.
* big refactor: make all crud functions for every db table, in common_services/database/tables/{table_the_funcs_are_for}
* rename types with similar names to db talbes, so ColorData from ml_analysis becomes PyColorData or something (look at how ml analysis ColorData is actually used)


# Features

* storage indicator bottom left, like googly photos
* albums
* front page -> 1 year ago, 4 years ago today, etc. in top balk
* photo trash bin?
* facial recognition
* upload photos
    * robust! stable!
* search photos
    * hybrid search
* photo map
    * time range restriction
* explore photos
    * cluster by photo embeddings
    * sort by all kinds of things (exposure, iso, hue, saturation, gps lat, lon, temperature, altitude, windyness (
      is_outdoor = true & sort by wind speed or gust))
    * group by: {country (if there are enough countries, otherwise group by province, otherwise group by city), camera
      model, main_subject, setting, animal type, pet type, food type, landmark, document type, photo_type, activity}
    * sunset/sunrise photos

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
- docker compose past beter in mn server setup
