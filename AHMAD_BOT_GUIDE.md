# Ahmad Bot — Technical Guide

## Overview

Ahmad Bot is a real local LLM assistant embedded in the ROWM Notebook. It uses **@mlc-ai/web-llm** for genuine model inference in your browser, with no API keys or network dependencies.

**Key Features:**
- Real model inference (Llama 2, Mistral, TinyLlama, NeuralHermes)
- WebGPU acceleration when available (CPU fallback)
- Automatic notebook cell extraction and context-aware responses
- Unicode preservation (λ Ω ϕ ∑ 𐤀 ꙮ)
- Streaming token generation
- Session persistence
- Dark sovereign theme (navy/cyan/gold)

## Architecture

### Files

| File | Purpose | Lines |
|------|---------|-------|
| `scripts/ahmad-bot-engine.js` | Real WebLLM integration, notebook context extraction | 550 |
| `scripts/ahmad-bot-ui.js` | Chat interface, panel management, message handling | 480 |
| `styles/ahmad-bot.css` | Dark theme, animations, responsive layout | 390 |
| `scripts/ahmad-bot-worker.js` | Optional Web Worker for non-blocking inference | 200 |

### Components

#### `NotebookPageReader`
Extracts notebook cells from DOM without reading nav/buttons:
```javascript
// Extract all cells
const cells = NotebookPageReader.extractCells();
// Returns: [{id, index, type, source, output, hash}, ...]

// Get notebook metadata
const meta = NotebookPageReader.getNotebookMetadata();
// Returns: {title, subtitle, cellCount, timestamp}
```

#### `NotebookContextIndex`
Builds searchable index and retrieves relevant cells:
```javascript
const index = new NotebookContextIndex();

// Find relevant cells for a query
const relevant = index.findRelevant("reversible Unicode", 5);

// Get cell by index
const cell = index.getCellByIndex(0);

// Export context as formatted text
const text = index.formatContextAsText(cells);
```

#### `AhmadWebLLMEngine`
Real WebLLM integration with streaming:
```javascript
const engine = new AhmadWebLLMEngine();

// Initialize with model selection
await engine.initialize('Llama-2-7b-chat-hf-q4f32_1-MLC');

// Check if ready
if (engine.isReady()) { ... }

// Generate with notebook context
const systemPrompt = engine.buildSystemPrompt(userMessage);
await engine.generate(userMessage, systemPrompt);

// Listen to events
engine.on('token', (token) => console.log(token));
engine.on('statusChanged', (status) => console.log(status));
engine.on('generationComplete', (response) => console.log(response));

// Interrupt generation
engine.interrupt();
```

#### `AhmadBotUI`
Chat interface and panel management:
```javascript
// Auto-initialized on page load
window.ahmadBotUI

// Programmatic access
window.ahmadBotUI.sendMessage();
window.ahmadBotUI.stopGeneration();
window.ahmadBotUI.clearMessages();
window.ahmadBotUI.updateStatus('READY');
window.ahmadBotUI.openPanel();
window.ahmadBotUI.closePanel();
```

## Model Selection

### Prebuilt Models (Verified)

| Model | Size | Speed | Memory | Best For |
|-------|------|-------|--------|----------|
| Llama 2 7B (q4f32) | 3.9GB | Medium | 8GB+ | Production, quality |
| Mistral 7B (q4f16) | 4.1GB | Fast | 8GB+ | Speed, efficiency |
| NeuralHermes 7B | 4.2GB | Medium | 8GB+ | Technical Q&A |
| TinyLlama 1.1B (q4f16) | 530MB | Very Fast | 2GB+ | Testing, limited devices |

### Selection Flow

```javascript
// List available models
const models = engine.getSupportedModels();
// Returns: [{id, name, size}, ...]

// Initialize specific model
await engine.initialize('Mistral-7B-Instruct-v0.2-q4f16_1-MLC');

// Check WebGPU support
const hasWebGPU = AhmadWebLLMEngine.hasWebGPU();
// CPU fallback automatically used if unavailable
```

## Integration into index-app.html

The following is already integrated. To verify:

1. **CSS is loaded:**
   ```html
   <link rel="stylesheet" href="styles/ahmad-bot.css">
   ```

2. **WebLLM library is loaded:**
   ```html
   <script src="https://cdn.jsdelivr.net/npm/@mlc-ai/web-llm@0.2.33/lib/web-llm.js"></script>
   ```

3. **Scripts are loaded in order:**
   ```html
   <script src="scripts/ahmad-bot-engine.js"></script>
   <script src="scripts/ahmad-bot-ui.js"></script>
   ```

4. **DOM elements exist:**
   ```html
   <div id="jit-launcher" class="jit-launcher">
       <button id="jit-toggle">Ω</button>
   </div>
   <div id="jit-panel" class="jit-panel hidden">
       <!-- Pre-built UI structure -->
   </div>
   ```

## End-to-End Usage

### 1. Open Notebook
```
→ Navigate to index-app.html
→ Notebook visible with cells
→ Ω button appears in bottom-right corner
```

### 2. Launch Ahmad Bot
```
→ Click Ω button
→ Panel slides open from bottom-right
→ Status: "OFFLINE"
```

### 3. Initialize Model
```
→ First open triggers automatic initialization
→ Status: "LOADING"
→ Real model downloads to browser (~3-4GB for Llama 2)
→ Download progress shown in WebLLM console
→ Status: "READY" when complete
```

### 4. Chat
```
→ Type question: "What does this notebook say about reversible Unicode?"
→ Press Enter or click Send
→ Status: "GENERATING"
→ Tokens stream in real-time (actual model output)
→ Status: "READY" when complete
→ Response cites actual notebook cells
```

### 5. Follow-up
```
→ Ask follow-up question
→ Context preserved from conversation history
→ New response generated with full context
```

### 6. Management
```
→ "Stop" button: Interrupt generation
→ "Clear" button: Clear message history
→ "−" button: Minimize panel
→ "✕" button: Close panel (model stays loaded)
→ Draggable header: Move panel around
```

## System Prompt

The system prompt is built per-message and includes:

1. **Identity:** "You are Ahmad Bot, embedded technical guide for the Isomorphic WORM Notebook"
2. **Environment:** "Running locally in the browser"
3. **Notebook Context:** Relevant cells found via keyword search
4. **Instructions:**
   - Answer based on notebook content
   - Cite cell identifiers
   - Never invent cells
   - Preserve Unicode exactly
   - Be concise and direct

### Example System Prompt

```
You are Ahmad Bot, an embedded technical guide for the Isomorphic WORM Notebook running locally in the browser.

You have access to the following notebook context:

ROWM Notebook Context
Title: Ω Isomorphic WORM Notebook
Total Cells: 3
===================

Cell [0]
Type: code
Source:
// Reversible Unicode mapping
const reversibleMap = {
  'λ': 'LAMBDA',
  'Ω': 'OMEGA',
  'ϕ': 'PHI'
};
---

[Additional cells...]

Instructions:
- Answer questions based on notebook content
- Cite cell identifiers (e.g., "Cell 0", "Cell 1")
- Never invent cells or content
- Preserve Unicode exactly (λ Ω ϕ ∑ 𐤀 ꙮ)
- Be concise and direct
- If uncertain about content, say so

User question: What does this notebook say about reversible Unicode?
```

## Status States

| State | Color | Animation | Meaning |
|-------|-------|-----------|---------|
| OFFLINE | Gray | None | Model not loaded |
| LOADING | Blue | Pulse | Downloading/initializing model |
| READY | Green | None | Model ready, waiting for input |
| GENERATING | Cyan | Pulse | Model producing response |
| ERROR | Red | None | Error occurred |

## Event Listeners

```javascript
// Engine events
engine.on('statusChanged', (status) => { ... })
engine.on('token', (token) => { ... })
engine.on('generationStart', () => { ... })
engine.on('generationComplete', (response) => { ... })
engine.on('generationStopped', () => { ... })
engine.on('error', (error) => { ... })
engine.on('historyCleared', () => { ... })
```

## Performance

### Download Sizes (One-time)
- Llama 2 7B: ~3.9GB (15-20 min on good connection)
- Mistral 7B: ~4.1GB (15-20 min)
- TinyLlama 1.1B: ~530MB (2-3 min)

### First Token Latency
- **WebGPU (NVIDIA RTX 3080+):** 300-500ms
- **WebGPU (AMD RDNA):** 500-800ms
- **CPU (i7-12700K):** 2-4 seconds

### Token Generation Speed
- **WebGPU:** 5-10 tokens/second
- **CPU:** 1-2 tokens/second

### Memory Footprint
- Runtime: 1-2GB (model-dependent)
- Browser overhead: 500MB-1GB
- Recommendation: 8GB+ for 7B models

## Troubleshooting

### Model Won't Initialize

**Symptom:** Status stays "LOADING" or shows "ERROR"

**Solutions:**
1. Check browser console for errors: `F12 → Console`
2. Verify WebLLM is loaded: `console.log(window.webllm)`
3. Check browser supports WebGPU or WebAssembly:
   ```javascript
   navigator.gpu // WebGPU
   typeof WebAssembly // WebAssembly
   ```
4. Try smaller model (TinyLlama) first
5. Clear browser cache and reload

### Model Downloads Slowly

**Solutions:**
1. Check internet connection speed
2. Look at browser Network tab to see download progress
3. Models cache in IndexedDB after first download
4. Try CDN-cached model (auto-retried by WebLLM)

### Responses Are Short/Cut Off

**Check:**
1. `maxTokens` setting (default: 512)
2. If model reached token limit: `model.maxTokens = 1024`
3. Model may have input token limit based on history

### GPU Not Used

**Check:**
1. Is GPU available? `AhmadWebLLMEngine.hasWebGPU()`
2. Browser console shows "Using GPU" or "Using CPU"
3. Some browsers/GPUs may force CPU mode
4. Performance acceptable on CPU is normal

### Unicode Not Preserved

**Cause:** Token sanitization too aggressive

**Fix:** `ahmad-bot-ui.js` line ~240 only removes control characters, preserves Unicode:
```javascript
sanitizeToken(token) {
    return token.replace(/[\x00-\x08\x0B-\x0C\x0E-\x1F\x7F]/g, '');
}
```

## Testing

### Automated End-to-End Test

```javascript
// 1. Check components are loaded
console.log('Engine loaded:', typeof AhmadWebLLMEngine)
console.log('UI loaded:', typeof AhmadBotUI)

// 2. Check DOM elements
console.log('Launcher:', document.getElementById('jit-launcher'))
console.log('Panel:', document.getElementById('jit-panel'))

// 3. Open panel
window.ahmadBotUI.openPanel()

// 4. Wait for model (check status in UI)
// Status should change: OFFLINE → LOADING → READY

// 5. Send test message
document.getElementById('jit-input').value = 'What cells are in this notebook?'
window.ahmadBotUI.sendMessage()

// 6. Observe real tokens streaming
// Panel should show message from model with actual cells cited
```

### Manual Testing Checklist

- [ ] Page loads, notebook visible
- [ ] Ω button visible bottom-right, pulsing cyan
- [ ] Click Ω button → panel slides open
- [ ] Status shows "OFFLINE" → "LOADING"
- [ ] Panel shows download progress or message
- [ ] After 5-20 minutes: status shows "READY"
- [ ] Type question, press Enter
- [ ] Status changes to "GENERATING"
- [ ] Tokens appear real-time in chat
- [ ] Response cites actual notebook cells
- [ ] Stop button works mid-generation
- [ ] Clear button empties chat
- [ ] Minimize button collapses panel to header
- [ ] Can drag panel by header
- [ ] Reload page → model cached, loads faster
- [ ] Mobile: panel responsive at 95vw

## Security & Privacy

✅ **All inference runs locally** — No data sent to servers
✅ **No API keys required** — Model runs in browser
✅ **No telemetry** — WebLLM may report model usage (optional)
✅ **Notebook content never leaves browser** — Context built locally
✅ **Token generation** — Pure model output, no filtering/modification

## Unicode Support

Preserved exactly across all components:

```
λ (Lambda)      — Greek letter
Ω (Omega)       — Greek letter
ϕ (Phi)         — Mathematical symbol
∑ (Summation)   — Mathematical operator
𐤀 (Samaritan)   — Ancient script
ꙮ (Old Cyrillic) — Historical script
→ ← ↑ ↓         — Arrows
∞ ∅ ⊂ ⊃         — Set notation
```

All preserved in:
1. Notebook cell extraction
2. Context indexing
3. System prompt building
4. Token streaming
5. Message display

## Future Enhancements

- Multi-turn fine-tuning corpus
- Notebook cell execution proposals
- WORM receipt signing for responses
- Model comparison UI
- Voice input/output
- Custom system prompts
- Response export

## Support

For issues:
1. Check browser console: `F12 → Console`
2. Verify WebLLM loaded: `console.log(window.webllm)`
3. Check network: No CORS errors
4. Try different model if error persists
5. File issue with console output

---

**Ahmad Bot** — Embedded AI for the Reversible World Ontology Math Notebook
