# Simple cross server album sharing

## User flow

1. Situation:
    * Alice has an album they want to share with Bob, the photos of the album are on Server A.
    * Bob wants these photos.
2. Alice clicks share album with other server
3. Alice gets an invite string: inv-{random_string}-alice@photos.alice.com
4. Alice sends the invite to Bob
5. Bob pastes the invite into his frontend UI.
6. Bob's server now downloads Alice's album (list of raw media files) to his server, and creates an album for them.
7. Bob's server ingests the new photos and assigns alice@photos.alice.com as owner of these photos.

## Implementation

### Endpoints:

#### Frontend:

- `GET` `/albums/{album_id}/generate-invite`
- `POST` `/albums/invite/check`, body: `{token: str}`
- `POST` `/albums/invite/accept`, body: `{token: str, name: str, description: str}`

#### S2S:

- `POST` `/s2s/albums/invite-summary`, body: `{token: str}`
- `POST` `/s2s/albums/files/{media_item_id}`, body: `{token: str}`

### Invite link generation [flow 3.]

GET `/albums/{album_id}/generate-invite`

* Check ownership
* generate random secure string
* put secure string in DB with following data (table: album_invites):
    * created_at
    * expires_at (set at created_at + settings.invitation_expiry_minutes)
    * secure string
    * album_id references album id
* return `inv-{random_string}-{user.name}@{settings.public_url}`

### Process invite [flow 5.]

Bob pastes Alice's invite string into his UI.

* The Bob's frontend calls: `/albums/invite/check`
    * Bob's server calls Alice's server: `/s2s/albums/invite-summary` with the token as body
        * Alice's server responds with a list of media_item_ids and an album `name` and `description`
    * Bob's server returns this information
* Bob's frontend shows a count of items, the album name, and description, and prompts Bob
  asking if they want to add this album to his server. The name and description are editable.
* If Bob says Accept, Bob's frontend calls `/albums/invite/accept`
    * Bobs server creates a job: `ImportAlbumFromToken` with payload: `{album_name, album_desc, token, album_owner}`
      `album_owner` should be alice@alice.photos.com
    * frontend receives a happy status code
* If bob says Decline, then no api calls are made, the invitation will expire.

#### Worker handler for `ImportAlbumFromToken`

* Bob's server calls `/s2s/albums/invite-summary` again, getting a list of media_item ids.
* An album is created with the name, description and app_user.id from the worker payload. Album owner is Bob.
* For each media item id a job is created: `ImportAlbumItem` with payload: `{album_id, token, album_owner}`

#### Worker handler for `ImportAlbumItem`

* Bob's server calls Alice's server at `/s2s/albums/files/{media_item_id}` {todo: where to put the token? can post
  request respond with bytes?}
    * Alice's server validates the token, checks that it matches the album, and then streams the raw file bytes in
      the response. {todo: Filename in a header?}
* Bob's server stores the received files in the media dir in a folder: media_dir()/{bob_folder}/{album_name}/{file}, if
  the `album_name` folder exists, append a number
* The watcher should automatically start processing the photos, but to be sure might as well enqueue the ingest job
  for them. [aside: i have to add a column to media_item: remote_owner: nullable text]
* A `pending_album_media_items` table will be needed to link the relative_paths that aren't ingested yet to the album
  and remote_owner, so that when
  they are done ingesting they are automatically put in the album, and the remote_owner is assigned.
* When an ingest is done, before inserting to db, check for `pending_album_media_items`, if exists, apply album and
  remote_owner. Then delete row from `pending_album_media_items`