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
