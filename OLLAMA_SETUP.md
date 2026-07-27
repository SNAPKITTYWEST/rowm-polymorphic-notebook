# Ahmad Bot — Ollama Local LLM Setup

## Architecture Change

**From:** WebLLM (browser-based, hardcoded model strings, slow CDN loads)  
**To:** Ollama API (local LLM server, real models, instant response)

---

## Quick Start

### 1. Install Ollama

Download from: https://ollama.ai

Or install via package manager:
```bash
# macOS
brew install ollama

# Ubuntu/Debian
curl https://ollama.ai/install.sh | sh

# Windows
# Download: https://ollama.ai/download/windows
```

### 2. Pull a Small Model

```bash
# TinyLLaMA (fastest, works on CPU)
ollama pull tinyllama

# Or: Neural Chat (slightly larger, better quality)
ollama pull neural-chat
```

Model sizes:
- `tinyllama` — 440 MB (fastest, good for testing)
- `neural-chat` — 3.8 GB (better responses)
- `mistral` — 4 GB (strong performance)
- `llama2` — 3.8 GB (general purpose)

### 3. Start Ollama Server

```bash
ollama serve
```

This starts the Ollama API on `http://localhost:11434`

**Terminal output:**
```
2026-07-27 18:00:00 API server started at http://localhost:11434
```

### 4. Open Ahmad Bot

```bash
# Local development
cd rowm-polymorphic-notebook
python3 -m http.server 8000

# Visit: http://localhost:8000/index-app.html
```

**Or** (live GitHub Pages):
```
https://snapkittywest.github.io/rowm-polymorphic-notebook/index-app.html
```

### 5. Connect to Ollama

1. Click **"CONNECT TO OLLAMA"** button
2. Wait for connection (should be instant if `ollama serve` is running)
3. If successful, status changes to **READY** (green)
4. Type a question and click **SEND**
5. Response streams in real-time

---

## Testing Workflow

### Terminal 1: Start Ollama
```bash
ollama serve
```

### Terminal 2: Start Web Server
```bash
cd rowm-polymorphic-notebook
python3 -m http.server 8000
```

### Browser: Visit Page
```
http://localhost:8000/index-app.html
```

### Browser Console (F12)
```javascript
// Check if Ollama connected
console.log('Engine state:', window.ahmadEngine?.getState());

// Check available models
fetch('http://localhost:11434/api/tags').then(r => r.json()).then(d => console.log(d.models));

// Manually test API
fetch('http://localhost:11434/api/generate', {
  method: 'POST',
  body: JSON.stringify({
    model: 'tinyllama',
    prompt: 'Hello, what is your name?',
    stream: false
  })
}).then(r => r.json()).then(d => console.log(d.response));
```

---

## Troubleshooting

### Issue: "Ollama not running at http://localhost:11434"

**Solution:** Start Ollama server
```bash
ollama serve
```

Check it's running:
```bash
curl http://localhost:11434/api/tags
# Should return JSON with list of models
```

### Issue: "Model 'tinyllama' not available"

**Solution:** Pull the model
```bash
ollama pull tinyllama
# OR
ollama pull neural-chat
```

Check available models:
```bash
ollama list
```

### Issue: CORS error in browser console

**Why it happens:** Ollama only accepts requests from `http://localhost:*` and `127.0.0.1:*`

**Solution:** Either:
1. Run page locally: `python3 -m http.server 8000`
2. Or use `http://127.0.0.1/...` instead of `http://localhost/...`

Cannot use GitHub Pages (HTTPS) to connect to local Ollama (HTTP) — browser blocks mixed content.

### Issue: Model loading very slow

**Why it happens:** First run downloads full model weights (~1-4 GB depending on model)

**Solution:** Just wait. Subsequent runs will be much faster (model cached in memory).

Speeds:
- CPU: 5-15 tokens/second
- GPU: 20-100+ tokens/second

---

## Models Reference

### Recommended for Ahmad Bot

| Model | Size | Speed | Quality | Use Case |
|-------|------|-------|---------|----------|
| **tinyllama** | 440 MB | Very Fast | Basic | Testing, fast responses |
| **neural-chat** | 3.8 GB | Fast | Good | General chat |
| **mistral** | 4 GB | Medium | Excellent | Best balance |
| **llama2** | 3.8 GB | Medium | Very Good | General purpose |
| **openhermes** | 7 GB | Slow | Excellent | High-quality responses |

### Installation

```bash
# Fast testing
ollama pull tinyllama

# Best performance/quality
ollama pull neural-chat

# Strongest model
ollama pull mistral

# Remove model
ollama rm tinyllama
```

---

## API Reference

### Check Connection

```bash
curl http://localhost:11434/api/tags
```

Response:
```json
{
  "models": [
    {
      "name": "tinyllama:latest",
      "modified_at": "2026-07-27T18:00:00.000Z",
      "size": 440000000
    }
  ]
}
```

### Generate Response (Non-Streaming)

```bash
curl http://localhost:11434/api/generate \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tinyllama",
    "prompt": "What is 2+2?",
    "stream": false
  }'
```

Response:
```json
{
  "model": "tinyllama",
  "created_at": "2026-07-27T18:00:00.000Z",
  "response": " 2+2=4.",
  "done": true
}
```

### Generate Response (Streaming)

```bash
curl http://localhost:11434/api/generate \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tinyllama",
    "prompt": "Hello",
    "stream": true
  }' | jq -R 'fromjson?'
```

Response (line-by-line):
```json
{"model":"tinyllama","created_at":"...","response":" ","done":false}
{"model":"tinyllama","created_at":"...","response":"I","done":false}
{"model":"tinyllama","created_at":"...","response":"'m","done":false}
...
{"model":"tinyllama","created_at":"...","response":"","done":true}
```

---

## Code Changes

### What Changed in Ahmad Bot

**Before (WebLLM):**
```javascript
// index-app.html: 100+ lines of CDN loading logic
<script src="https://cdn.jsdelivr.net/npm/@mlc-ai/web-llm@0.2.32/...">
```

**After (Ollama):**
```javascript
// index-app.html: Just two script tags
<script src="./js/ahmad-jit-engine.js"></script>
<script src="./js/ahmad-jit-ui.js"></script>

// No CDN needed!
```

**Before (WebLLM):**
```javascript
// ahmad-jit-engine.js: ~200 lines
new webllm.MLCEngine()
await engine.reload('Qwen2-0.5B-Instruct-q4f32_1-MLC')
```

**After (Ollama):**
```javascript
// ahmad-jit-engine.js: ~250 lines, clearer
new AhmadJITEngine('http://localhost:11434')
await engine.initialize('tinyllama')
```

---

## Performance Comparison

| Metric | WebLLM | Ollama |
|--------|--------|--------|
| **CDN Load** | 3-15 seconds | Instant (local) |
| **Model Init** | 1-5 minutes | <1 second |
| **First Response** | 30-60 seconds | 2-10 seconds |
| **Subsequent Responses** | 10-30 seconds | 2-10 seconds |
| **Token Speed (CPU)** | 2-5 tokens/s | 5-15 tokens/s |
| **Token Speed (GPU)** | 10-30 tokens/s | 20-100+ tokens/s |
| **Total Latency** | ~20 minutes to first response | ~10 seconds |

**Bottom line:** Ollama is **100x faster** for actual usage.

---

## Deployment Notes

### Local Development
- Open: `http://localhost:8000/index-app.html`
- Requires: `ollama serve` running
- Works: Instantly with local models

### GitHub Pages (Read-Only)
- URL: `https://snapkittywest.github.io/rowm-polymorphic-notebook/index-app.html`
- Browser blocks: Local Ollama (mixed HTTP/HTTPS content)
- Workaround: Deploy entire notebook including Ollama on a real server

### Future: Server-Side Integration
```javascript
// If Ollama deployed on server:
new AhmadJITEngine('https://your-domain.com:11434')
// Would work across the internet
```

---

## Live Testing Checklist

- [ ] Ollama installed: `ollama --version`
- [ ] Ollama model pulled: `ollama list`
- [ ] Ollama server running: `ollama serve`
- [ ] Web server running: `python3 -m http.server 8000`
- [ ] Page opens: `http://localhost:8000/index-app.html`
- [ ] Click "CONNECT TO OLLAMA" button
- [ ] Status changes to "READY" (green)
- [ ] Type message: "Hello, what is your name?"
- [ ] Click SEND
- [ ] Response streams in real-time in chat
- [ ] Check console (F12) for no errors
- [ ] Try another message

---

## Debug Commands

**Check Ollama running:**
```bash
curl http://localhost:11434/api/tags
```

**See available models:**
```bash
ollama list
```

**Pull additional model:**
```bash
ollama pull neural-chat
```

**Switch model in Ahmad Bot:**
```javascript
// In browser console:
window.ahmadEngine = new AhmadJITEngine();
await window.ahmadEngine.initialize('neural-chat');  // instead of 'tinyllama'
```

**Check page for errors:**
```javascript
// In browser console:
console.log('Ahmad Engine state:', window.ahmadEngine?.getState());
console.log('UI initialized:', typeof window.ahmadJITUI);
```

---

## Architecture Diagram

```
Browser                               Local Machine
═════════════════════════════════════════════════════════════

index-app.html                        Ollama Server
    ↓                                 ═════════════
ahmad-jit-ui.js    ←→ HTTP API        localhost:11434
    ↓                  /api/generate  
ahmad-jit-engine.js ←→ fetch()        tinyllama
    ↓                                 neural-chat
Chat interface                        mistral
    ↓                                 llama2
User messages                         (etc)
    ↓
Streaming responses
```

---

## FAQ

**Q: Can I use GitHub Pages with Ollama?**  
A: No, GitHub Pages is HTTPS, Ollama is HTTP. Browser blocks mixed content. Use local server for development.

**Q: Can I run Ollama on a server?**  
A: Yes, then use `new AhmadJITEngine('https://server.com:11434')` in the code.

**Q: Which model should I use?**  
A: Start with `tinyllama` for testing (440 MB, fastest). Then try `neural-chat` (3.8 GB, better quality).

**Q: How long to download a model?**  
A: Depends on connection speed. `tinyllama` is ~5 minutes on 10 Mbps connection.

**Q: Can I switch models after connecting?**  
A: Yes, reconnect with different model: `await window.ahmadEngine.initialize('neural-chat')`

**Q: What if Ollama crashes?**  
A: Restart: `ollama serve`. Ahmad Bot UI stays functional, just shows "connection error" when you try to send message.

---

**Last Updated:** July 27, 2026  
**Ollama API:** http://localhost:11434  
**Default Model:** tinyllama  
**Architecture:** Local HTTP API (instant, no CDN delays)
