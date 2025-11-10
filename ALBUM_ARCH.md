# S2S Federation Design Document

## 1. Overview

This document outlines the design for a generic, direct Server-to-Server (S2S) federation system. The goal is to
establish a secure and reliable communication layer between independent instances of the application, initiated by a
user's intent to share a resource (initially, an album).

The design prioritizes a simplified user experience by combining the server-pairing (peer setup) and the resource
invitation into a single, token-based action. The underlying architecture remains generic to support future federated
features.

**Core Principles:**

* **Action-Initiated Federation:** Servers learn about each other and establish trust only when a user initiates a
  share. There is no manual pre-configuration of peers.
* **Generic Event Bus:** Once connected, servers communicate via a generic "event" structure that is not tied to any
  specific feature.
* **Secure:** Communication is protected against tampering and replay attacks.
* **Reliable:** The system handles network failures and offline servers via a persistent job queue and state
  reconciliation.

## 2. Server Setup & User Identity

#### 2.1. Environment Configuration (`.env`)

The following variables are critical for federation and must be configured by the self-hoster.

```dotenv
# The public-facing base URL of this server instance, without a trailing slash.
# This URL is shared with other servers so they know where to send events.
# MUST be the address reachable from the public internet.
# Example: https://my-photos.example.com
SERVER_PUBLIC_URL="http://localhost:3000"
```

#### 2.2. User Identity

* The concept of a `federated_user_id` will be standardized as `username@domain`.
* The `domain` part will be derived from the `SERVER_PUBLIC_URL`.
* The user's `name` must be unique per server instance.

## 3. Database Schema Changes

#### 3.1. `app_user` table (Modification)

A unique `name` field is required for the federated identity.

#### 3.2. `federation_invitations` table (New)

Stores short-lived, single-use tokens that bootstrap the peer connection and initial share.

```sql
CREATE TABLE federation_invitations
(
    id                 UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    -- The one-time-use token given to the user.
    token_hash         TEXT        NOT NULL UNIQUE,
    -- The pre-generated secret that will be shared with the peer upon claiming.
    shared_secret      TEXT        NOT NULL,
    -- The user who created the invitation.
    inviter_user_id    INTEGER     NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    -- JSONB blob to store context, e.g., the album being shared.
    resource_payload   JSONB       NOT NULL,
    -- The peer server that successfully claimed this invite.
    claimed_by_peer_id UUID        REFERENCES peer_servers (id) ON DELETE SET NULL,
    expires_at         TIMESTAMPTZ NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

#### 3.3. `peer_servers` table (New)

Stores information about trusted peer servers after an invitation is successfully claimed.

```sql
CREATE TABLE peer_servers
(
    id            UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    hostname      TEXT        NOT NULL UNIQUE, -- e.g., https://photos.friend.com
    shared_secret TEXT        NOT NULL,        -- Secret for authenticating messages
    friendly_name TEXT,
    is_enabled    BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

#### 3.4. `outgoing_federated_events` table (New)

A persistent queue and log for events that need to be sent.

```sql
CREATE TABLE outgoing_federated_events
(
    id             UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    peer_server_id UUID        NOT NULL REFERENCES peer_servers (id) ON DELETE CASCADE,
    payload        JSONB       NOT NULL,                   -- The full JSON payload of the event
    status         TEXT        NOT NULL DEFAULT 'pending', -- 'pending', 'sent', 'failed'
    last_attempt   TIMESTAMPTZ,
    retry_count    INTEGER     NOT NULL DEFAULT 0,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_outgoing_federated_events_status_retry ON outgoing_federated_events (status, last_attempt);
```

#### 3.5. `incoming_federated_events` table

(New)
Stores nonces of successfully processed events to prevent replays. Can be periodically cleaned.

```sql
CREATE TABLE incoming_federated_events
(
    nonce          TEXT PRIMARY KEY,
    peer_server_id UUID        NOT NULL REFERENCES peer_servers (id) ON DELETE CASCADE,
    processed_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_incoming_federated_events_processed_at ON incoming_federated_events (processed_at);
```

## 4. Architectural Changes

#### 4.1. Federation Library Crate (`crates/libs/federation`)

This new crate will house all non-API-specific federation logic:

* **`FederationService`:** High-level service for creating invitations and enqueueing outgoing events.
* **`InvitationService`:** Logic for creating, validating, and claiming invitation tokens.
* **`Event` Structs:** Typed structs for the generic event envelope and specific payloads (e.g.,
  `AlbumInvitation`, `AlbumMediaItemAdded`).
* **`PeerClient`:** An HTTP client wrapper responsible for signing and sending requests to peers.

#### 4.2. API Routes & Middleware (`crates/binaries/api/src/routes/events/`)

The S2S API will live under `/federation/`.

* **`S2SAuthMiddleware`:** An Axum middleware that will protect the `/federation/receive` endpoint. It will perform HMAC
  signature verification and nonce checking before passing the request to the handler.
* The `api` crate will now have a dependency on the new `federation` crate.

#### 4.3. Worker Modifications (`crates/binaries/worker/`)

* A new job type `SendFederatedEvent` is added.
* The worker will have a handler for this job that uses the `PeerClient` from the `federation` crate to send the event
  and handle retries.

## 5. API Endpoint Specification

#### 5.1. `POST /federation/invitations/claim`

This is a public, unauthenticated endpoint used by a remote server to claim a one-time invitation token.

* **Description:** A server (Server B) calls this endpoint on another server (Server A) to exchange a one-time token for
  the necessary details to establish a peer relationship and process an initial resource share.
* **Request Body:**
  ```json
  {
    "invitation_token": "the_long_random_token_string",
    "claiming_server_url": "https://server-b.com",
    "claiming_user_username": "jane"
  }
  ```
* **Success Response (200 OK):**
  ```json
  {
    "shared_secret": "the_new_secret_for_server_b_to_talk_to_a",
    "inviter": {
      "federated_id": "john@server-a.com",
      "name": "John Doe"
    },
    "resource_payload": {
      "type": "album",
      "album": {
        "id_on_sender": "uuid-for-album-on-server-a",
        "name": "Vacation 2025"
      }
    }
  }
  ```
* **Error Responses:**
    * `404 Not Found`: The invitation token does not exist, has expired, or has already been used.
    * `400 Bad Request`: Invalid request body.

#### 5.2. `POST /federation/receive`

This is the main, secured webhook for receiving all ongoing federated events after the peer connection is established.

* **Description:** A trusted peer sends events to this endpoint. The `S2SAuthMiddleware` protects it.
* **Middleware Logic:**
    1. Extract the raw request body.
    2. Get the `X-Signature-256` header.
    3. Get the sender's identity. This can be done by requiring the sender to identify itself with another header, e.g.,
       `X-Sender-URL: https://server-a.com`.
    4. Look up the `peer_server` record using the `X-Sender-URL`.
    5. Compute the HMAC signature and compare it. Fail with `403 Forbidden` if it doesn't match.
    6. Decode the json and check the timestamp. Fail if it's over 3 minutes old.
    7. Check the `nonce` from JSON. Fail with `409 Conflict` if it's a replay.
    8. If all checks pass, attach the validated `peer_server` and the event payload to the request extensions and pass
       it to the handler.
* **Request Body (Generic Event Envelope):**
  ```json
  {
    "event_type": "PHOTO_ADDED_TO_ALBUM",
    "nonce": "a-unique-random-uuid-v4",
    "timestamp": "2025-11-10T20:00:00Z",
    "payload": {
      // ... feature-specific data ...
    }
  }
  ```
* **Success Response (202 Accepted):** An immediate `202` response acknowledges receipt. The actual processing happens
  in the background.

## 6. Detailed Flows

#### 6.1. Flow: The Combined Invitation & Peer Setup

This flow details how Person A on Server A invites Person B on Server B to an album, establishing the peer connection in
the process.

**Part 1: Generating the Invite (Server A)**

1. **UI:** Person A is on their album page and clicks "Share" -> "Create Federated Invite".
2. **API (Server A):** The frontend calls an endpoint like `POST /federation/{album_id}/invitations`.
3. **Backend (`FederationService` on Server A):**
    * Generates a cryptographically secure `invitation_token` (e.g., 64 random bytes, base64 encoded).
    * Hashes the `invitation_token` (e.g., with SHA256) and stores the hash in the `federation_invitations` table. The
      raw token is only shown to the user once.
    * Generates a new, secure `shared_secret` for this future relationship.
    * Creates a `resource_payload` JSON object: `{"type": "album", "album": {"id": "album_uuid", "name": "..."}}`.
    * Saves a new record in `federation_invitations` with the `token_hash`, `shared_secret`, `inviter_user_id`,
      `resource_payload`, and an `expires_at` (e.g., 24 hours from now).
4. **UI:** The API returns an **Invite String** to the UI, which is then displayed to Person A. The string is a
   combination of the raw `invitation_token` and the server's public URL:
   `inv-xxxxxxxxxxxxxxxxx@my-photos.example.com`

**Part 2: Claiming the Invite (Server B)**

5. **Out-of-Band:** Person A copies the Invite String and sends it to Person B via a chat app or email.
6. **UI:** Person B navigates to a "Join Album" page in their app and pastes the full Invite String.
7. **API (Server B):** The frontend parses the string to get the token (`inv-xxxxxxxx`) and the hostname (
   `my-photos.example.com`). It then makes a request to its own backend.
8. **Backend (`InvitationService` on Server B):**
    * Makes a `POST` request to `https://my-photos.example.com/federation/invitations/claim`. The request body includes
      the token, its own public URL (from its `.env`), and Person B's username.
9. **Backend (Server A receives claim):**
    * The `/claim` handler hashes the received token and looks it up in `federation_invitations`.
    * It validates the token (exists, not expired, not used).
    * It creates a new `peer_servers` record for Server B using the `claiming_server_url` and the `shared_secret` stored
      with the invitation.
    * It marks the invitation as `claimed_by_peer_id`.
    * It returns a `200 OK` with the `shared_secret` (a different one, for B to talk to A), inviter details, and the
      `resource_payload`.
10. **Backend (`InvitationService` on Server B receives response):**
    * It now has everything it needs to trust Server A. It creates a new `peer_servers` record for Server A using the
      hostname from the invite string and the `shared_secret` from the response body. The connection is now established
      in both directions.
    * It parses the `resource_payload`. Seeing it's an album, it creates a new local "mirror" album for Person B.
    * It adds the inviter (`john@my-photos.example.com`) to the `album_collaborator` table for this new album.
    * It creates an in-app notification for Person B: "You now have access to the shared album 'Vacation 2025'".

**Part 3: Finalizing the Invitation**

11. To confirm the process and let the original server know the user has seen the invite, Server B's backend enqueues a
    new federated event of type `ALBUM_INVITATION_CONFIRMED` to be sent to Server A. This tells Server A it can begin
    sending updates about the album.

#### 6.2. Flow: Ongoing Federation (Photo Added)

This demonstrates how the established connection is used.

1. **Action (Server A):** Person A adds a new photo to the shared album.
2. **Backend (`AlbumService` on Server A):**
    * After saving the photo-album link locally, it checks for federated collaborators. It finds `jane@server-b.com`.
    * It calls `FederationService::enqueue_event()`, targeting the peer record for `server-b.com`, with an event type
      `PHOTO_ADDED_TO_ALBUM` and a payload containing photo metadata and URLs.
3. **Worker (Server A):** The worker picks up the job, retrieves the event payload and the peer's `shared_secret`, signs
   the request, and `POST`s it to `https://server-b.com/federation/receive`.
4. **Backend (Server B):**
    * The `S2SAuthMiddleware` validates the request.
    * The event router sees the `event_type` and passes it to the `AlbumEventHandler`.
    * The handler adds the remote photo's metadata to Person B's view of the album. Person B now sees the new photo.

## 7. Reliability & State Reconciliation

#### 7.1. Handling Offline Servers

* The worker's retry logic handles temporary outages. If Server B is down, the job will fail and be retried later with
  exponential backoff. The event remains in the `outgoing_federated_events` table until successfully delivered.

#### 7.2. State Reconciliation

To handle long-term outages and ensure consistency, a periodic "catch-up" task is needed.

* **Mechanism:** A scheduled job runs daily on each server.
* **Process:**
    1. The job on Server A iterates through its peers. For Server B, it sends a `STATE_RECONCILIATION_PING` event.
    2. Server B receives this event and responds by sending back a `STATE_RECONCILIATION_PONG` event. The payload of
       this "pong" event contains the `nonce` of the very last event it successfully processed from Server A.
    3. Server A receives the pong. It looks up the received `nonce` in its `outgoing_federated_events` table. If it
       finds any events that were sent *after* that nonce, it assumes they were lost and re-enqueues them by changing
       their status back to `pending`.