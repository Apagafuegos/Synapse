# Development Guide

## New Development Workflow (Production-like)

LogLens now uses Axum to serve the frontend in both development and production for consistency.

### Option 1: Using Helper Script (Recommended)

```bash
cd loglens-web
./dev.sh
```

This will:
1. Start Vite build watcher (rebuilds on file changes)
2. Start Axum backend server
3. Serve everything on http://localhost:3000

### Option 2: Manual Process

**Terminal 1 - Frontend Build Watcher:**
```bash
cd loglens-web/frontend-react
npm run dev:watch
```

**Terminal 2 - Axum Backend:**
```bash
cd loglens-web
export DATABASE_URL="sqlite:data/loglens.db"
cargo run
```

Access the application at: http://localhost:3000

### Option 3: Traditional Vite Dev Server (Legacy)

If you prefer the Vite dev server with HMR:

**Terminal 1 - Vite Dev Server:**
```bash
cd loglens-web/frontend-react
npm run dev
```

**Terminal 2 - Axum Backend:**
```bash
cd loglens-web
export DATABASE_URL="sqlite:data/loglens.db"
LOGLENS_PORT=3001 cargo run
```

Access via Vite proxy at: http://localhost:3000

## Development Experience Changes

### What's Different

- **Build Time:** ~1-2 second delay on file changes (vs instant HMR)
- **Page Refresh:** Manual browser refresh needed after changes
- **Consistency:** Development now matches production behavior exactly

### What's the Same

- ✅ All API routes work (`/api/*`)
- ✅ WebSocket connections (`/ws`)
- ✅ File uploads
- ✅ SPA routing (all routes serve index.html)
- ✅ Static asset serving
- ✅ CORS configuration

## Troubleshooting

### "Cannot find frontend files"

Ensure you've built the frontend at least once:
```bash
cd loglens-web/frontend-react
npm run build
```

### "Port 3000 already in use"

Check for running processes:
```bash
lsof -i :3000
kill -9 <PID>
```

### "Database not found"

Ensure DATABASE_URL is set correctly:
```bash
export DATABASE_URL="sqlite:data/loglens.db"
```

Or create the data directory:
```bash
mkdir -p loglens-web/data
```

### Changes not appearing

1. Check that build watcher is running (Terminal 1)
2. Wait for build to complete (~1-2 seconds)
3. Hard refresh browser (Ctrl+Shift+R or Cmd+Shift+R)
4. Check console for build errors

## Production Build

### Local Production Build

Build production-ready artifacts:

```bash
cd loglens-web/frontend-react
npm run build
```

Run production server:
```bash
cd loglens-web
export DATABASE_URL="sqlite:data/loglens.db"
export LOGLENS_FRONTEND_DIR="frontend-react/dist"
cargo run --release
```

### Docker Production Build

Build and run with Docker Compose (recommended for production):

```bash
# From the project root
cd loglens
docker-compose up -d
```

This will:
1. Build WASM module
2. Build React frontend (static build, no Vite dev server)
3. Build Axum backend
4. Serve everything on http://localhost:3000

The Docker setup uses the same Axum-only serving method as local development for consistency.

**Docker Commands:**
```bash
# Build and start
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down

# Rebuild after changes
docker-compose up -d --build

# Check health
docker-compose ps
curl http://localhost:3000/health
```
