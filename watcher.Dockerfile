# =====================================================================
# Stage 1: Python Base (Required for workspace dependencies like pyo3)
# =====================================================================
FROM python:3.12-slim-bookworm AS python-base
ENV PYTHONUNBUFFERED=1

# =====================================================================
# Stage 2: Builder Base (Rust Toolchain + System Deps)
# =====================================================================
FROM python-base AS builder-base

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev libpq-dev protobuf-compiler nasm \
    curl git \
    zlib1g-dev libbz2-dev libreadline-dev libsqlite3-dev \
    libncursesw5-dev xz-utils tk-dev libxml2-dev libxmlsec1-dev libffi-dev liblzma-dev \
    && rm -rf /var/lib/apt/lists/*

# -- Rust Toolchain Installation --
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# -- Install cargo-chef --
RUN cargo install cargo-chef

# =====================================================================
# Stage 3: Planner (for Rust dependency caching)
# =====================================================================
FROM builder-base AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =====================================================================
# Stage 4: Builder (Compiles watcher binary)
# =====================================================================
FROM builder-base AS builder
WORKDIR /app

# -- Rust Dependency Caching Layer --
COPY --from=planner /app/recipe.json recipe.json
COPY .sqlx .sqlx
RUN cargo chef cook --release --recipe-path recipe.json

# -- Build Watcher Application --
COPY . .
RUN cargo build --release -p watcher

# =====================================================================
# Stage 5: Runner (Lean runtime image)
# =====================================================================
FROM debian:bookworm-slim AS runner

# Install runtime dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy runtime assets.
COPY config/settings.yaml ./config/
COPY migrations migrations
COPY --from=builder /app/target/release/watcher .

CMD ["./watcher"]
