# Multi-stage Dockerfile for LogLens
# Builds WASM, React frontend, and Rust backend in a single optimized image

# Stage 1: Build WASM module
FROM rustlang/rust:nightly-bookworm AS wasm-builder

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /build

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members (required for workspace to work)
COPY loglens-core ./loglens-core
COPY loglens-wasm ./loglens-wasm
COPY loglens-web ./loglens-web

# Build WASM package
WORKDIR /build/loglens-wasm
RUN wasm-pack build --target web --out-dir pkg --release

# Stage 2: Build React frontend
FROM node:20-bookworm AS frontend-builder

WORKDIR /build

# Copy WASM build output
COPY --from=wasm-builder /build/loglens-wasm/pkg ./loglens-wasm/pkg

# Copy package files for dependency installation
COPY loglens-web/frontend-react/package.json loglens-web/frontend-react/package-lock.json ./loglens-web/frontend-react/
WORKDIR /build/loglens-web/frontend-react

# Install dependencies
RUN npm ci

# Copy config files needed for build (one at a time to avoid path issues)
COPY loglens-web/frontend-react/tsconfig.json ./
COPY loglens-web/frontend-react/tsconfig.node.json ./
COPY loglens-web/frontend-react/vite.config.ts ./
COPY loglens-web/frontend-react/tailwind.config.js ./
COPY loglens-web/frontend-react/postcss.config.js ./

# Copy source files and assets
COPY loglens-web/frontend-react/index.html ./
COPY loglens-web/frontend-react/src ./src

# Build frontend (skip WASM rebuild since we already have it)
RUN npm run build:skip-wasm

# Stage 3: Build Rust backend
FROM rust:1.90.0-bookworm AS backend-builder
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./

# Copy all crates
COPY loglens-core ./loglens-core
COPY loglens-wasm ./loglens-wasm
COPY loglens-web ./loglens-web

# Copy sqlx offline data for compile-time query verification
COPY .sqlx ./.sqlx

# Enable sqlx offline mode to avoid database connection during build
ENV SQLX_OFFLINE=true

# Build backend in release mode
RUN cargo build --release --bin loglens-web

# Stage 4: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /build/target/release/loglens-web ./loglens-web

# Copy frontend build
COPY --from=frontend-builder /build/loglens-web/frontend-react/dist ./frontend-react/dist

# Copy migrations
COPY loglens-web/migrations ./migrations

# Create directories for data persistence
RUN mkdir -p /app/data /app/uploads && \
    chmod 755 /app/data /app/uploads

# Set environment variables
ENV DATABASE_URL=sqlite:/app/data/loglens.db
ENV PORT=3000
ENV RUST_LOG=info
ENV LOGLENS_FRONTEND_DIR=/app/frontend-react/dist
ENV LOGLENS_UPLOAD_DIR=/app/uploads

# Expose port
EXPOSE 3000

# Set working directory
WORKDIR /app

# Run the application
CMD ["./loglens-web"]
