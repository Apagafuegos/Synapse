# Multi-stage Dockerfile for Synapse
# Builds WASM, React frontend, and Rust backend in a single optimized image

# Stage 1: Build WASM module
FROM rustlang/rust:nightly-bookworm AS wasm-builder

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /build

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members (required for workspace to work)
COPY synapse-core ./synapse-core
COPY synapse-wasm ./synapse-wasm
COPY synapse-web ./synapse-web
COPY synapse-cli ./synapse-cli
COPY synapse-mcp ./synapse-mcp

# Build WASM package
WORKDIR /build/synapse-wasm
RUN wasm-pack build --target web --out-dir pkg --release

# Stage 2: Build React frontend
FROM node:20-bookworm AS frontend-builder

WORKDIR /build

# Copy WASM build output to the correct location relative to frontend
COPY --from=wasm-builder /build/synapse-wasm/pkg ./synapse-wasm/pkg

# Copy package files for dependency installation
COPY synapse-web/frontend-react/package.json synapse-web/frontend-react/package-lock.json ./synapse-web/frontend-react/
WORKDIR /build/synapse-web/frontend-react

# Install dependencies
RUN npm ci

# Copy config files needed for build
COPY synapse-web/frontend-react/tsconfig.json ./
COPY synapse-web/frontend-react/tsconfig.node.json ./
COPY synapse-web/frontend-react/vite.config.ts ./
COPY synapse-web/frontend-react/tailwind.config.js ./
COPY synapse-web/frontend-react/postcss.config.js ./

# Copy source files and assets
COPY synapse-web/frontend-react/index.html ./
COPY synapse-web/frontend-react/src ./src

# Build frontend (skip WASM rebuild since we already have it)
RUN npm run build:skip-wasm

# Stage 3: Build Rust backend
FROM rust:1.90.0-bookworm AS backend-builder
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./

# Copy all crates
COPY synapse-core ./synapse-core
COPY synapse-wasm ./synapse-wasm
COPY synapse-web ./synapse-web
COPY synapse-cli ./synapse-cli
COPY synapse-mcp ./synapse-mcp

# Copy sqlx offline data for compile-time query verification
COPY .sqlx ./.sqlx

# Enable sqlx offline mode to avoid database connection during build
ENV SQLX_OFFLINE=true

# Build backend in release mode
RUN cargo build --release --bin synapse-web

# Stage 4: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 curl wkhtmltopdf && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /build/target/release/synapse-web ./synapse-web

# Copy frontend build
COPY --from=frontend-builder /build/synapse-web/frontend-react/dist ./frontend-react/dist

# Copy migrations
COPY synapse-web/migrations ./migrations

# Create directories for data persistence
RUN mkdir -p /app/data /app/uploads && \
    chmod 755 /app/data /app/uploads

# Set environment variables
ENV DATABASE_URL=sqlite:/app/data/synapse.db
ENV PORT=3000
ENV RUST_LOG=info
ENV LOGLENS_FRONTEND_DIR=/app/frontend-react/dist
ENV LOGLENS_UPLOAD_DIR=/app/uploads

# Expose port
EXPOSE 3000

# Set working directory
WORKDIR /app

# Run the application
CMD ["./synapse-web"]
