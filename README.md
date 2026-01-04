# Photos Backend

Backend for **Ruurd Photos**, a self-hosted Google Photos alternative. Handles the API, media ingestion, classification,
and search.

## Features

* Photo and video ingestion
* ML-based analysis (tagging, embeddings, facial recognition)
* REST API for frontend integration
* Hybrid semantic/text search
* File system watcher for new media

## Prerequisites

* **nasm**: `winget install -e --id NASM.NASM`
* **protoc**: `winget install -e --id Google.Protobuf`
* **sqlx**: `cargo install sqlx-cli`
* **Python** installed and added to `PATH` (e.g., `C:\Users\YourName\AppData\Local\Programs\Python\Python312` on
  Windows; Linux support needs testing)
* **uv** installed for setting up the virtualenv in `ml_analysis`
* **Rust** to compile the backend
* **Postgres** database set up

## Installation

### 1. Clone the repo

```bash
git clone https://github.com/RuurdBijlsma/photos-backend.git
cd photos-backend
```

### 2. Set up `ml_analysis` environment

```bash
cd crates/libs/ml_analysis/py_ml
uv sync
```

### 3. Set environment variables

```text
APP__DATABASE__URL=postgres://user:pass@localhost/photos
APP__AUTH__JWT_SECRET=your123secret
```

### 4. Set up database

*Make sure postgres is running and the env variables are set*

To apply the migrations, setting up the database structure:

```bash
sqlx migrate run
```

### 5. (Optional) Configure settings

Edit `config/settings.yaml` to adjust backend settings.

---

## Usage

### 1. Run integration tests

```shell
cargo test -p test_integration -- --nocapture
```

### 2. Clippy

```shell
cargo clippy --no-deps --all-features -- -D clippy::all -D clippy::pedantic -D clippy::nursery
```

### 1. Run the backend crates

There are 4 crates required for full backend functionality:

1. `crates/binaries/api` – Web API
2. `crates/binaries/watcher` – Watches media directories and enqueues jobs for created/deleted files
3. `crates/binaries/worker` – Processes jobs (generates thumbnails, analyzes metadata, updates database)

Run each crate in a separate terminal:

```bash
cargo run -p api
cargo run -p watcher
cargo run -p worker
```

> Tip: You can run multiple workers simultaneously to speed up ingestion.

### 2. Run the frontend

1. Clone the
   frontend: [https://github.com/RuurdBijlsma/photos-frontend](https://github.com/RuurdBijlsma/photos-frontend)
2. Follow the frontend instructions to run it
3. Access the application