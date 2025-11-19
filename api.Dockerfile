# =====================================================================
# Stage 1: Builder
# =====================================================================
FROM python:3.12-slim-bullseye AS builder

# -- Base Setup --
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

# -- Python Layer --
WORKDIR /usr/src/app
ENV VENV_PATH="/usr/src/app/crates/libs/ml_analysis/py_ml/.venv"
ENV PATH="${VENV_PATH}/bin:${PATH}"
RUN pip install uv
COPY crates/libs/ml_analysis/py_ml/pyproject.toml crates/libs/ml_analysis/py_ml/uv.lock ./crates/libs/ml_analysis/py_ml/
WORKDIR /usr/src/app/crates/libs/ml_analysis/py_ml
RUN uv sync --no-cache

# -- Rust Dependency Caching Layer --
WORKDIR /usr/src/app
RUN mkdir -p crates/libs/common_types/proto
COPY Cargo.toml Cargo.lock ./
COPY crates/apps/api/Cargo.toml ./crates/apps/api/
COPY crates/apps/tasks/Cargo.toml ./crates/apps/tasks/
COPY crates/apps/watcher/Cargo.toml ./crates/apps/watcher/
COPY crates/apps/worker/Cargo.toml ./crates/apps/worker/
COPY crates/libs/app_state/Cargo.toml ./crates/libs/app_state/
COPY crates/libs/common_services/Cargo.toml ./crates/libs/common_services/
COPY crates/libs/common_types/Cargo.toml crates/libs/common_types/build.rs ./crates/libs/common_types/
COPY crates/libs/common_types/proto/photos.proto ./crates/libs/common_types/proto/
COPY crates/libs/generate_thumbnails/Cargo.toml ./crates/libs/generate_thumbnails/
COPY crates/libs/ml_analysis/Cargo.toml ./crates/libs/ml_analysis/
COPY crates/test_integration/Cargo.toml ./crates/test_integration/

# Create a dummy main.rs files for each app.
# todo: 2 RUN for dir in foo bar; do echo $dir; done
RUN mkdir -p crates/apps/api/src && \
    echo "fn main() {}" > crates/apps/api/src/main.rs
RUN mkdir -p crates/apps/tasks/src && \
    echo "fn main() {}" > crates/apps/tasks/src/main.rs
RUN mkdir -p crates/apps/watcher/src && \
    echo "fn main() {}" > crates/apps/watcher/src/main.rs
RUN mkdir -p crates/apps/worker/src && \
    echo "fn main() {}" > crates/apps/worker/src/main.rs
# Create a dummy lib.rs files for each lib
RUN mkdir -p crates/libs/app_state/src && \
    echo "" > crates/libs/app_state/src/lib.rs
RUN mkdir -p crates/libs/common_services/src && \
    echo "" > crates/libs/common_services/src/lib.rs
RUN mkdir -p crates/libs/common_types/src && \
    echo "" > crates/libs/common_types/src/lib.rs
RUN mkdir -p crates/libs/generate_thumbnails/src && \
    echo "" > crates/libs/generate_thumbnails/src/lib.rs
RUN mkdir -p crates/libs/ml_analysis/src && \
    echo "" > crates/libs/ml_analysis/src/lib.rs
RUN mkdir -p crates/test_integration/src && \
    echo "" > crates/test_integration/src/lib.rs

# -- Build Dependencies --
RUN cargo build --release # build dependencies

# -- Build libs --
COPY crates/libs crates/libs
COPY .sqlx .sqlx
RUN cargo build --release # build libs

# -- Build api --
COPY crates/apps/api/src crates/apps/api/src
RUN cargo build --release --package api # build api


# =====================================================================
# Stage 2: Runner
# =====================================================================
FROM python:3.12-slim-bullseye AS runner

# Install runtime dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create a non-root user for security.
RUN addgroup --system app && adduser --system --ingroup app app

# Copy necessary runtime assets from the host.
COPY config/settings.yaml ./config/

# Copy the Python virtual environment, which contains installed packages for 'ml_analysis'.
COPY --from=builder /usr/src/app/crates/libs/ml_analysis/py_ml/.venv ./.venv

# Copy the compiled binary from the 'builder' stage.
COPY --from=builder /usr/src/app/target/release/api .

# Set correct permissions for all application files.
RUN chown -R app:app .

# Switch to the non-root user.
USER app

# Add the venv's bin directory to the PATH. This ensures the application
# uses the Python interpreter and packages from the venv.
ENV PATH="/app/.venv/bin:${PATH}"

# Expose the port the API server will listen on.
EXPOSE 9475

# Set the command to run the application.
CMD ["./api"]