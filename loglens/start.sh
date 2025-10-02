#!/bin/bash

# LogLens Unified Startup Script
# Builds and starts the complete application (WASM + Frontend + Backend)

set -e  # Exit on error

echo "🚀 Starting LogLens build and deployment..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Build WASM module
echo -e "${YELLOW}📦 Building WASM module...${NC}"
cd loglens-wasm
wasm-pack build --target web --out-dir pkg --release
cd ..
echo -e "${GREEN}✓ WASM build complete${NC}"

# Step 2: Build React frontend
echo -e "${YELLOW}🎨 Building React frontend...${NC}"
cd loglens-web/frontend-react

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "📥 Installing frontend dependencies..."
    npm install
fi

# Build frontend for production
npm run build
cd ../..
echo -e "${GREEN}✓ Frontend build complete${NC}"

# Step 3: Build backend (if --build flag is provided)
if [[ "$1" == "--build" ]] || [[ "$1" == "-b" ]]; then
    echo -e "${YELLOW}⚙️  Building backend in release mode...${NC}"
    cargo build --release --bin loglens-web
    echo -e "${GREEN}✓ Backend build complete${NC}"
    BACKEND_BIN="./target/release/loglens-web"
else
    echo -e "${YELLOW}⚙️  Using development build (use --build for release)${NC}"
    BACKEND_BIN="cargo run --bin loglens-web --"
fi

# Step 4: Create data and uploads directories if they don't exist
mkdir -p data uploads

# Step 5: Set environment variables
export PORT=3000
export DATABASE_URL=sqlite:./data/loglens.db

# Step 6: Start the backend server
echo -e "${GREEN}🎉 Build complete! Starting LogLens server...${NC}"
echo -e "${GREEN}📍 Server will be available at http://localhost:3000${NC}"
echo ""

# Run the backend
if [[ "$1" == "--build" ]] || [[ "$1" == "-b" ]]; then
    $BACKEND_BIN
else
    cargo run --bin loglens-web
fi
