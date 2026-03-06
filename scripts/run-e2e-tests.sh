#!/bin/bash
set -e

# Internal script to run E2E tests inside the container

# Navigate to repository root (parent of scripts directory)
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Cleanup function to kill spawned processes
cleanup() {
    echo "🧹 Cleaning up processes..."
    [ -n "$FRONTEND_PID" ] && kill $FRONTEND_PID 2>/dev/null || true
    [ -n "$MOCK_WS_PID" ] && kill $MOCK_WS_PID 2>/dev/null || true
}
trap cleanup EXIT

echo "🔧 Setting up test environment..."

# 0. Kill any stale Vite/bun processes from previous runs
echo "🧹 Cleaning up stale processes..."
pkill -9 -f "vite.*--port 5173" 2>/dev/null || true
pkill -9 -f "bun run.*5173" 2>/dev/null || true
pkill -9 -f "node.*vite.*5173" 2>/dev/null || true
sleep 2

# 1. Ensure bun is installed (needed for building and running UI)
if ! command -v bun &> /dev/null; then
    echo "❌ bun not found in PATH."
    exit 1
fi

# 2. Start mock WebSocket server
echo "🚀 Starting Mock WebSocket server..."
cat << 'EOF' > /tmp/mock-ws-server.mjs
import fs from 'fs';

const server = Bun.serve({
  port: 8000,
  tls: {
    key: Bun.file('temp/certs/server.key.pem'),
    cert: Bun.file('temp/certs/server.cert.pem'),
  },
  async fetch(req, server) {
    const url = new URL(req.url);
    if (url.pathname === '/health') {
      return new Response('OK');
    }
    if (req.method === 'POST' && url.pathname === '/api/internal/publish') {
      const body = await req.text();
      try {
        const parsed = JSON.parse(body);
        if (parsed.channel) {
          global.lastMessages = global.lastMessages || new Map();
          global.lastMessages.set(parsed.channel, body);
        }
      } catch (e) {}
      server.publish('broadcast', body);
      return new Response('OK');
    }
    if (url.pathname === '/ws') {
      const upgraded = server.upgrade(req);
      if (upgraded) return;
      return new Response('Upgrade failed', { status: 400 });
    }
    return new Response('Not found', { status: 404 });
  },
  websocket: {
    open(ws) {
      ws.subscribe('broadcast');
      if (global.lastMessages) {
        for (const msg of global.lastMessages.values()) {
          ws.send(msg);
        }
      }
    },
    message(ws, message) {},
    close(ws) {},
  }
});
console.log('Mock WS server running on 8000');
EOF

# Generate self-signed certs for testing if missing
mkdir -p temp/certs
if [ ! -f "temp/certs/server.cert.pem" ] || [ ! -r "temp/certs/server.key.pem" ]; then
    echo "🔐 Generating self-signed certificates..."
    rm -f temp/certs/server.cert.pem temp/certs/server.key.pem
    openssl req -newkey rsa:2048 -nodes -keyout temp/certs/server.key.pem -x509 -days 365 -out temp/certs/server.cert.pem -subj "/CN=localhost" 2>/dev/null
    chmod 644 temp/certs/server.key.pem temp/certs/server.cert.pem
fi

bun run /tmp/mock-ws-server.mjs > /tmp/mock-ws.log 2>&1 &
MOCK_WS_PID=$!

echo "⏳ Waiting for Mock WS Server..."
for i in {1..30}; do
    if curl -k -s https://localhost:8000/health > /dev/null; then
        echo "✅ Mock WS Server is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Mock WS Server failed to start."
        cat /tmp/mock-ws.log
        kill $MOCK_WS_PID || true
        exit 1
    fi
    sleep 1
done

# 4. Serve the Frontend
echo "🌐 Starting Frontend Dev Server..."
cd src/ui

# Install dependencies if needed (container might not have node_modules)
if [ ! -d "node_modules" ]; then
    echo "📦 Installing UI dependencies..."
    bun install
fi

# Check for permission issues with dist directory or its subdirectories
if [ -d "dist" ]; then
    if [ ! -w "dist" ] || find dist -type d ! -writable 2>/dev/null | grep -q .; then
        echo "❌ Error: dist directory has wrong permissions (likely created by root)"
        echo "   Please run: sudo rm -rf src/ui/dist"
        kill $MOCK_WS_PID || true
        exit 1
    fi
fi

# Build the frontend for preview mode (eliminates Vite dev optimization issues)
# Note: Using default base path (/) for preview server, not /static for production backend
echo "🏗️  Building frontend..."
# Faster polling for E2E tests
export VITE_RECONNECTION_POLL_INTERVAL_MS=500
export VITE_NEW_IP_POLL_INTERVAL_MS=500
export VITE_REBOOT_TIMEOUT_MS=2000
export VITE_FACTORY_RESET_TIMEOUT_MS=2000
export VITE_FIRMWARE_UPDATE_TIMEOUT_MS=2000

if bun run build-preview > /tmp/vite-build.log 2>&1; then
    echo "✅ Frontend build complete!"
else
    echo "❌ Frontend build failed!"
    cat /tmp/vite-build.log
    kill $MOCK_WS_PID || true
    exit 1
fi

# Start Vite preview server (serves production build)
echo "🚀 Starting Vite preview server..."
export VITE_HTTPS=true
bun run preview --port 5173 > /tmp/vite.log 2>&1 &
FRONTEND_PID=$!

# Wait for preview server
echo "⏳ Waiting for preview server..."
for i in {1..30}; do
    if curl -k -s https://localhost:5173 > /dev/null; then
        echo "✅ Preview server is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Preview server failed to start."
        cat /tmp/vite.log
        kill $FRONTEND_PID || true
        kill $MOCK_WS_PID || true
        exit 1
    fi
    sleep 1
done

# 5. Run Playwright Tests
echo "🧪 Running Playwright Tests..."

# Check for permission issues with Playwright test results
if [ -d "test-results" ] && [ ! -w "test-results" ]; then
    echo "❌ Error: Playwright test-results directory has wrong permissions (likely created by root)"
    echo "   Please run: sudo rm -rf src/ui/test-results src/ui/playwright-report"
    kill $FRONTEND_PID || true
    kill $MOCK_WS_PID || true
    exit 1
fi

# Install Playwright browsers (always run to ensure correct version)
echo "📦 Ensuring Playwright browsers are installed..."
npx playwright install chromium

# BASE_URL is set for playwright.config.ts
export BASE_URL="https://localhost:5173"

# Run tests
npx playwright test --reporter=list "$@"

TEST_EXIT_CODE=$?

exit $TEST_EXIT_CODE
