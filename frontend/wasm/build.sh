#!/bin/bash
# Build WASM modules for ROWM Notebook

set -e

echo "🔧 Building WASM engine for ROWM Notebook..."

# Install wasm-pack if not present
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.org/wasm-pack/installer/init.sh -sSf | sh
fi

# Build for bundler target (wasm-pack default)
echo "📦 Compiling Unicode Engine..."
wasm-pack build \
    --target bundler \
    --out-dir ../dist/unicode-engine \
    -- --features unicode-engine

echo "📦 Compiling Crypto Engine..."
wasm-pack build \
    --target bundler \
    --out-dir ../dist/crypto-engine \
    -- --features crypto-engine

# Optional: Build for browser/nodejs targets
echo "📦 Building for browser (optional)..."
wasm-pack build \
    --target web \
    --out-dir ../dist/browser \
    2>/dev/null || echo "⚠️  Browser target skipped"

echo ""
echo "✅ WASM build complete!"
echo ""
echo "Output locations:"
echo "  - Bundler: ../dist/unicode-engine/"
echo "  - Bundler: ../dist/crypto-engine/"
echo "  - Web: ../dist/browser/"
echo ""
echo "Next: Import in JavaScript:"
echo "  import init, * as rowm from './dist/unicode-engine/index.js';"
echo "  await init();"
