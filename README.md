# Photos Backend

Backend for **Ruurd Photos**, a self-hosted Google Photos alternative. Handles the api, media ingestion, classification,
and search.

## Features

* Photo and video ingestion
* ML-based analysis (e.g., tagging, embeddings, facial recognition)
* REST API for frontend integration
* Hybrid (semantic/text) search through photos
* Watch filesystem for new photos and videos

## Prerequisites

* **Python** installed and added to `PATH` (e.g., `C:\Users\YourName\AppData\Local\Programs\Python\Python312` on
  Windows) (Linux needs some testing).
* **uv** installed, to set up the virtualenv for the `ml_analysis` sub-crate.
* **Rust** to compile the backend.
* **Postgres** set up for database.

## Installation

### 1. Get the repo

```bash
git clone https://github.com/RuurdBijlsma/photos-backend.git
cd photos-backend
```

### 2. Set up ml_analysis

```bash
cd crates/ml_analysis/py_ml
uv sync
```

### 3. Set environment variables:

1. `APP__DATABASE__URL=postgres://user:pass@localhost/photos`
2. `APP__AUTH__JWT_SECRET=your123secret`

### 4. Run it

There are 4 crates that need to run for full functionality of the backend:

1. `crates/api` -> The web API
2. `crates/indexer` -> Scans the media directory, and enqueues ingest/remove jobs to sync the db with the file system.
3. `crates/watcher` -> Watches the media directory, and handles created/deleted files by queueing up ingest/remove jobs.
4. `crates/worker` -> Watch for jobs and process them. An ingest job means generating thumbnails, analyzing the file for
   metadata, and putting that metadata in the database. A remove job deletes everything related to the file from db and
   thumbnails directory.

Run these crates with the following command:
*each command in a separate terminal because they don't quit*

```bash
cargo run -p api
cargo run -p indexer
cargo run -p watcher
cargo run -p worker
```

Sidenote: you can run multiple workers to process jobs simultaneously, this can speed up ingestion.

### 5. Run the frontend

1. Get the frontend from: https://github.com/RuurdBijlsma/photos-frontend
2. Follow the instructions from there to run it.
3. Use the application

### (optional) Configure settings

Change values in `config/settings.yaml` to change settings.

## Usage

Run your backend service as needed for ingestion and API access. ML analysis scripts require Python in PATH.

---

If you want, I can also make a slightly **catchier “one-paragraph intro” version** that’s more GitHub-friendly while
keeping it short. Do you want me to do that?
