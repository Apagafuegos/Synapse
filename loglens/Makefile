# LogLens Build System

.PHONY: all clean build-wasm build-frontend build-backend dev

# Build everything
all: build-wasm build-backend

# Clean all build artifacts
clean:
	cargo clean
	cd loglens-wasm && rm -rf pkg
	cd loglens-web/frontend-react && npm run clean || true

# Build WASM module
build-wasm:
	cd loglens-wasm && wasm-pack build --target web --out-dir pkg

# Build WASM module for development (faster, with debug info)
build-wasm-dev:
	cd loglens-wasm && wasm-pack build --target web --out-dir pkg --dev

# Build backend
build-backend:
	cargo build --release

# Build frontend (React)
build-frontend: build-wasm
	cd loglens-web/frontend-react && npm install && npm run build

# Development mode - build WASM in dev mode and start backend
dev: build-wasm-dev
	cargo run --bin loglens-web

# Install frontend dependencies
install-frontend:
	cd loglens-web/frontend-react && npm install

# Run frontend dev server (requires backend to be running)
dev-frontend:
	cd loglens-web/frontend-react && npm run dev

# Run tests
test:
	cargo test
	cd loglens-wasm && wasm-pack test --node
	cd loglens-web/frontend-react && npm test

# Full development setup
setup: build-wasm install-frontend
	@echo "Development setup complete!"
	@echo "Run 'make dev' to start the backend server"
	@echo "Run 'make dev-frontend' in another terminal to start the frontend"

# Production build - builds everything for production deployment
build-prod: build-wasm build-frontend build-backend
	@echo "✓ Production build complete!"
	@echo "Run 'make start-prod' or './start.sh --build' to start the server"

# Start production server (after build-prod)
start-prod:
	@echo "Starting LogLens in production mode..."
	@mkdir -p data uploads
	@export PORT=3000 DATABASE_URL=sqlite:./data/loglens.db && ./target/release/loglens-web

# Unified start - builds and runs everything (recommended for production)
start: build-wasm build-frontend
	@echo "Starting LogLens (development build)..."
	@mkdir -p data uploads
	@export PORT=3000 DATABASE_URL=sqlite:./data/loglens.db && cargo run --bin loglens-web