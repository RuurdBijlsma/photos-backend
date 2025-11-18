# =====================================================================
# Stage 1: Builder
#
# This stage starts with a Python base image, adds the Rust toolchain,
# and then builds the final application binary.
# =====================================================================
FROM python:3.12-slim-bullseye AS builder

# -- Base Setup --
# Install system dependencies needed for both Python (native extensions) and Rust.
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev libpq-dev protobuf-compiler nasm \
    curl git \
    zlib1g-dev libbz2-dev libreadline-dev libsqlite3-dev \
    libncursesw5-dev xz-utils tk-dev libxml2-dev libxmlsec1-dev libffi-dev liblzma-dev \
    && rm -rf /var/lib/apt/lists/*

# -- Rust Toolchain Installation --
# Install Rust using rustup, the official toolchain installer.
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# -- Python Dependency Layer --
# Install uv, the fast Python package manager.
RUN pip install uv

# This layer is only invalidated if pyproject.toml or uv.lock changes.
WORKDIR /usr/src/app/crates/libs/ml_analysis/py_ml
COPY crates/libs/ml_analysis/py_ml/pyproject.toml crates/libs/ml_analysis/py_ml/uv.lock ./

# Create the virtual environment and sync dependencies.
# This step is slow, so we want it to be cached as often as possible.
# todo: i can probably remove this
# todo: make other dockerfiles and try to extract common logic.
ENV PATH="/usr/src/app/crates/libs/ml_analysis/py_ml/.venv/bin:${PATH}"
RUN uv sync --no-cache

# -- Rust Build Layer --
# This layer is invalidated whenever any application source code changes.
WORKDIR /usr/src/app
# Copy the rest of the project source code
# todo: only copy what's needed for rust build
COPY . .

# Build the 'api' application in release mode.
# The Rust compiler will find the Python interpreter via the PATH.
RUN cargo build --release --package api

# =====================================================================
# Stage 2: Runner
#
# This stage is completely UNCHANGED. It creates the final, lightweight
# image by copying the compiled binary and the Python venv from the builder.
# =====================================================================
FROM python:3.12-slim-bullseye AS runner

# Install runtime dependencies. libpq-dev is needed for postgres client libs.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Create a non-root user for security purposes
RUN addgroup --system app && adduser --system --ingroup app app

# Copy necessary runtime assets
COPY config/settings.yaml ./config/

# Copy the Python virtual environment, which contains your installed packages.
COPY --from=builder /usr/src/app/crates/libs/ml_analysis/py_ml/.venv ./.venv

# Copy the compiled binary from the 'builder' stage
COPY --from=builder /usr/src/app/target/release/api .

# Set correct permissions for all application files
RUN chown -R app:app .

# Switch to the non-root user
USER app

# Add the venv's bin directory to the PATH. This ensures that your application
# uses the python interpreter and packages from the venv.
ENV PATH="/app/.venv/bin:${PATH}"

# Expose the port the API server will listen on
EXPOSE 9475

# Set the command to run the application
CMD ["./api"]