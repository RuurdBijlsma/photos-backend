# =====================================================================
# Stage 1: Python Base (Shared by Runner)
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
# Stage 3: Python Dependencies (Needs Builder Base for compilation)
# =====================================================================
FROM builder-base AS python-deps
WORKDIR /app

RUN pip install uv
COPY crates/libs/ml_analysis/py_ml/pyproject.toml crates/libs/ml_analysis/py_ml/uv.lock ./crates/libs/ml_analysis/py_ml/
WORKDIR /app/crates/libs/ml_analysis/py_ml
RUN uv sync --no-cache

# =====================================================================
# Stage 4: Planner
# =====================================================================
FROM builder-base AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =====================================================================
# Stage 5: Builder (Rust)
# =====================================================================
FROM builder-base AS builder
WORKDIR /app

# -- Rust Dependency Caching Layer --
COPY --from=planner /app/recipe.json recipe.json
COPY .sqlx .sqlx
RUN cargo chef cook --release --recipe-path recipe.json

# -- Build Application --
COPY . .
RUN cargo build --release --package api

# =====================================================================
# Stage 6: Runner
# =====================================================================
FROM python-base AS runner

# Install runtime dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    libimage-exiftool-perl \
    curl \
    file \
    libgl1 \
    libglib2.0-0 \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy necessary runtime assets from the host.
COPY config/settings.yaml ./config/
COPY migrations migrations
COPY crates/libs/ml_analysis/py_ml ./py_ml

# Copy the Python virtual environment from python-deps
COPY --from=python-deps /app/crates/libs/ml_analysis/py_ml/.venv ./py_ml/.venv

# Copy the compiled binary from the 'builder' stage.
COPY --from=builder /app/target/release/api .

# Add the venv's bin directory to the PATH. This ensures the application
# uses the Python interpreter and packages from the venv.
ENV PATH="/app/py_ml/.venv/bin:${PATH}"
ENV APP_PY_ML_DIR="/app/py_ml"

# Set the command to run the application.
CMD ["./api"]