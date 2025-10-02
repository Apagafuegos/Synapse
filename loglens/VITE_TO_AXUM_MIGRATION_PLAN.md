# Migration Plan: Vite Dev Server → Axum Production Serving

**Generated:** 2025-10-02
**Estimated Effort:** 1-1.5 hours
**Risk Level:** Very Low (95% confidence)
**Breaking Changes:** None

---

## Executive Summary

**Great news:** Your production setup already uses Axum to serve the React frontend! No backend code changes needed. This is a **workflow migration**, not a framework migration.

**Framework Choice:** **Axum** ✅
- Already integrated and working perfectly
- Meets all requirements: low overhead, fast, light
- Zero migration effort on backend

---

## Current State Analysis

### Production Setup (Already Correct) ✅

**Location:** `loglens-web/src/main.rs:159-163`

```rust
.nest_service(
    "/",
    ServeDir::new(&frontend_path)
        .not_found_service(ServeFile::new(&index_path))
)
```

**What's working:**
- ✅ Axum serves static files via `tower-http::ServeDir`
- ✅ SPA routing via `ServeFile::not_found_service(index.html)`
- ✅ Serves on port 3000, handles `/api` and `/ws` routes
- ✅ Docker builds React app and serves through Axum
- ✅ CORS configured correctly
- ✅ WebSocket support enabled
- ✅ File upload handling

### Development Setup (Needs Alignment)

**Current workflow:**
```bash
# Terminal 1: Vite dev server (port 3000)
cd loglens-web/frontend-react
npm run dev

# Terminal 2: Axum backend (port 3001)
cd loglens-web
cargo run
```

**Issues:**
- Dependency on Vite dev server
- Development behavior differs from production
- Proxy configuration needed (vite.config.ts lines 54-64)
- Custom SPA fallback middleware (vite.config.ts lines 8-35)

---

## Migration Plan

### Phase 1: Update Frontend Scripts

**File:** `loglens-web/frontend-react/package.json`

**Changes:**
Add new development scripts to the `scripts` section:

```json
{
  "scripts": {
    "dev": "vite",
    "dev:watch": "vite build --watch --mode development",
    "dev:build": "vite build --mode development",
    "build": "npm run build:wasm && tsc && vite build",
    // ... rest of scripts
  }
}
```

**Rationale:**
- `dev:watch`: Continuous rebuild on file changes (recommended for development)
- `dev:build`: Single development build
- Keep existing `dev` script for those who prefer Vite dev server

**Estimated Time:** 5 minutes

---

### Phase 2: Create Development Helper Script (Optional)

**File:** `loglens-web/dev.sh` (new)

**Content:**
```bash
#!/bin/bash

# Development helper script for LogLens
# Runs frontend build watcher and Axum backend concurrently

set -e

echo "🚀 Starting LogLens development environment"
echo ""

# Kill background processes on exit
trap 'kill $(jobs -p) 2>/dev/null' EXIT

# Start frontend build watcher in background
echo "📦 Starting frontend build watcher..."
cd frontend-react
npm run dev:watch &
FRONTEND_PID=$!
cd ..

# Wait for initial build
echo "⏳ Waiting for initial frontend build..."
sleep 5

# Start Axum backend
echo "🦀 Starting Axum backend server..."
echo "📍 Server will be available at http://localhost:3000"
echo ""
cargo run

# This will run until Ctrl+C
wait
```

**Make executable:**
```bash
chmod +x loglens-web/dev.sh
```

**Estimated Time:** 10 minutes

---

### Phase 3: Update Documentation

**File:** Create `loglens-web/DEVELOPMENT.md` or update existing README

**Content:**

```markdown
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
```

**Estimated Time:** 15 minutes

---

### Phase 4: Testing & Validation

**Checklist:**

#### Build System
- [ ] Build frontend successfully: `npm run dev:build`
- [ ] Build watcher works: `npm run dev:watch` (verify rebuilds on file change)
- [ ] Production build works: `npm run build`

#### Backend Server
- [ ] Axum starts without errors
- [ ] Serves on port 3000
- [ ] Logs show frontend directory path correctly
- [ ] Database initializes properly

#### SPA Routing
- [ ] Navigate to http://localhost:3000 → shows homepage
- [ ] Navigate to http://localhost:3000/projects → shows projects page
- [ ] Navigate to http://localhost:3000/dashboard → shows dashboard
- [ ] Refresh on nested route (e.g., /projects/123) → page loads correctly (not 404)
- [ ] Browser back/forward buttons work

#### API Endpoints
- [ ] GET /api/projects → returns data
- [ ] POST /api/projects → creates project
- [ ] GET /api/dashboard/stats → returns statistics
- [ ] GET /api/health → returns health status
- [ ] Error responses formatted correctly

#### WebSocket
- [ ] WebSocket connects on /ws
- [ ] Real-time updates work (if applicable)
- [ ] Connection remains stable

#### File Operations
- [ ] File upload works
- [ ] Uploaded files accessible via API
- [ ] File deletion works

#### Static Assets
- [ ] JavaScript bundles load (check browser DevTools Network tab)
- [ ] CSS styles apply correctly
- [ ] Images load (favicon, logos, etc.)
- [ ] Fonts load if applicable
- [ ] No 404 errors in browser console

#### Development Experience
- [ ] Change frontend file → build watcher triggers
- [ ] Refresh browser → changes visible
- [ ] Build completes in reasonable time (~1-2s)
- [ ] Error messages clear and helpful

#### Production Parity
- [ ] Behavior identical to production Docker setup
- [ ] No unexpected errors or warnings
- [ ] Performance acceptable

**Estimated Time:** 30 minutes

---

## Behavior Guarantees

### What Remains Unchanged ✅

| Feature | Status |
|---------|--------|
| SPA routing (all routes → index.html) | ✅ Preserved |
| API routes (`/api/*`) | ✅ Preserved |
| WebSocket (`/ws`) | ✅ Preserved |
| Static asset serving | ✅ Preserved |
| CORS configuration | ✅ Preserved |
| File upload functionality | ✅ Preserved |
| Database operations | ✅ Preserved |
| Cache system | ✅ Preserved |
| Circuit breaker | ✅ Preserved |
| Logging/tracing | ✅ Preserved |

### Development Experience Trade-offs

| Aspect | Before (Vite Dev) | After (Axum Watch) |
|--------|-------------------|-------------------|
| Hot Module Replacement | ✅ Instant | ❌ No HMR |
| Page Refresh | Auto | Manual |
| Rebuild Speed | Instant | 1-2 seconds |
| Production Parity | Different | ✅ Identical |
| Server Processes | 2 (Vite + Axum) | 1 (Axum only) |
| Port Configuration | Proxy setup | Direct |
| Mental Model | Complex (proxy) | Simple |
| Debugging | Mixed dev/prod | Pure production |

---

## Files Modified

### Modified Files

1. **loglens-web/frontend-react/package.json**
   - Add `dev:watch` and `dev:build` scripts
   - No dependencies changed

### New Files

2. **loglens-web/dev.sh** (optional)
   - Development helper script
   - Make executable with `chmod +x`

3. **loglens-web/DEVELOPMENT.md** (recommended)
   - Comprehensive development guide
   - Troubleshooting section

### Unchanged Files ✅

- **loglens-web/src/main.rs** - Already correct!
- **loglens-web/src/config.rs** - Already correct!
- **loglens-web/src/routes.rs** - No changes needed
- **Dockerfile** - Already correct!
- **docker-compose.yml** - Already correct!
- **All other backend files** - No changes needed

---

## Why Axum (Not Actix)?

### Decision Matrix

| Criterion | Axum | Actix |
|-----------|------|-------|
| **Already Integrated** | ✅ Yes | ❌ No (requires full rewrite) |
| **Low Overhead** | ✅ Yes (hyper + tower) | ✅ Yes |
| **Fast** | ✅ Yes | ✅ Slightly faster (~5-10%) |
| **Light** | ✅ Yes | ✅ Yes |
| **Migration Effort** | 1-1.5 hours | 20-30 hours |
| **Risk** | Very Low | Medium |
| **Static File Serving** | ✅ tower-http | ✅ actix-files |
| **Ecosystem** | ✅ Active | ✅ Mature |
| **Learning Curve** | ✅ Already known | New patterns |

### Recommendation: **Axum** ✅

**Rationale:**
- Production already uses Axum successfully
- Meets all requirements (low overhead, fast, light)
- Zero backend code changes needed
- Minimal migration effort
- True production parity in development
- Actix's 5-10% performance edge irrelevant for this use case

**When to consider Actix:**
- Building new project from scratch
- Need absolute maximum performance (> 100K req/s)
- Team has Actix expertise
- Willing to invest 20-30 hours in migration

**Current situation:** Axum is the clear choice.

---

## Rollback Plan

If issues arise, instantly rollback:

```bash
# Use old development workflow
cd loglens-web/frontend-react
npm run dev  # Terminal 1

cd loglens-web
LOGLENS_PORT=3001 cargo run  # Terminal 2
```

No code changes need reverting since backend wasn't modified.

---

## Estimates Summary

| Phase | Task | Time | Risk |
|-------|------|------|------|
| 1 | Update package.json scripts | 5 min | None |
| 2 | Create dev helper script (optional) | 10 min | None |
| 3 | Update documentation | 15 min | None |
| 4 | Testing & validation | 30 min | Low |
| **Total** | **Complete migration** | **1-1.5 hours** | **Very Low** |

**Confidence Level:** 95%

**Risk Factors:**
- ✅ Backend code already correct (no changes = no risk)
- ✅ Production setup already proven
- ✅ Minimal new code introduced
- ✅ Easy rollback if issues found
- ⚠️ Development experience slightly different (acceptable trade-off)

---

## Post-Migration

### Update Team

- [ ] Notify team of new development workflow
- [ ] Update onboarding documentation
- [ ] Create demo video (optional)
- [ ] Update CI/CD if affected

### Future Enhancements (Optional)

1. **Live Reload**
   - Add `tower-livereload` crate for auto browser refresh
   - Trades "low overhead" for better DX
   - Recommend only if team requests

2. **Development TLS**
   - Add `axum-tls` for HTTPS in development
   - Useful for testing secure contexts
   - Not needed for most use cases

3. **Better Build Feedback**
   - Add visual feedback when build completes
   - Browser notification or terminal bell
   - Minimal effort improvement

4. **Concurrent Build Tool**
   - Replace shell script with `cargo-watch` or similar
   - Better process management
   - More robust than shell script

---

## Conclusion

This is a **low-effort, high-value migration** that brings true production parity to development. The backend is already perfect—we're just aligning the development workflow.

**Key Takeaway:** You're not migrating frameworks; you're simplifying your development workflow to match what production already does successfully.

**Next Steps:**
1. Review this plan
2. Schedule 2-hour migration window
3. Execute phases 1-4
4. Validate thoroughly
5. Update team documentation
6. Close Vite dev server chapter ✅
