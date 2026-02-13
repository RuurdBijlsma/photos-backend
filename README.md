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
* **sqlx**: `cargo install sqlx-cli`
* **llama.cpp**: Required for LLM-based image categorization, OCR, and quality
  judging. [Installation Guide](https://github.com/ggml-org/llama.cpp/blob/master/docs/install.md). Exact command for llama-server found in `./scripts/setup_env.ps1`.
* **Python** installed and added to `PATH` (e.g., `C:\Users\YourName\AppData\Local\Programs\Python\Python312` on
  Windows; Linux support needs testing)
* **uv** installed for setting up the virtualenv in `ml_analysis`
* **Rust** to compile the backend
* **Postgres** database set up with `pgvector` installed, docker/podman command for this available in `./scripts/start_postgres.ps1`.

## Installation

### 1. Clone the repo

```bash
git clone https://github.com/RuurdBijlsma/photos-backend.git
cd photos-backend
```

### 2.

### 2. Set up `ml_analysis` environment

```bash
cd crates/libs/ml_analysis/py_ml
uv sync
```

### 3. Start the LLM Server

The worker requires a running `llama-server` to perform visual analysis. It is recommended to use a Vision-Language
model like Qwen3-VL.

```bash
llama-server -hf unsloth/Qwen3-VL-4B-Instruct-GGUF:Q4_K_M --n-gpu-layers 99 --jinja --top-p 0.8 --temp 0.7 --min-p 0.0 --flash-attn on --presence-penalty 1.5 --ctx-size 8192 --models-max 1 --sleep-idle-seconds 60
```

*Note: Ensure the `llm_base_url` in `config/settings.yaml` matches your server address (default
is `http://localhost:8080`).*

### 4. Set environment variables

```text
APP__DATABASE__URL=postgres://user:pass@localhost/photos
APP__AUTH__JWT_SECRET=your123secret
```

### 5. Set up database

*Make sure postgres is running and the env variables are set*

To apply the migrations, setting up the database structure:

```bash
sqlx migrate run
```

### 6. (Optional) Configure settings

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

### 3. Run the backend crates

There are 3 binaries required for full backend functionality:

1. `crates/apps/api` – Web API
2. `crates/apps/watcher` – Watches media directories and enqueues jobs for created/deleted files
3. `crates/apps/worker` – Processes jobs (generates thumbnails, analyzes metadata, updates database)

Run each crate in a separate terminal:

```bash
cargo run -p api
cargo run -p watcher
cargo run -p worker
```

> Tip: You can run multiple workers simultaneously to speed up ingestion.

### 4. Run the frontend

1. Clone the
   frontend: [https://github.com/RuurdBijlsma/photos-frontend](https://github.com/RuurdBijlsma/photos-frontend)
2. Follow the frontend instructions to run it
3. Access the application