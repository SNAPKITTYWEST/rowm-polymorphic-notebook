# WebLLM CDN Loading Test Guide

## Quick Diagnostic (Copy-Paste to Browser Console)

### Test 1: Check if WebLLM is loaded
```javascript
console.log('window.webllm:', typeof window.webllm);
console.log('window.webllmReady:', window.webllmReady);
console.log('window.webllmError:', window.webllmError);
console.log('window.webllmCDNLoading:', window.webllmCDNLoading);
```

**Expected Output:**
- `window.webllm: object` (if loaded)
- `window.webllmReady: true` (if successfully loaded)
- `window.webllmError: null` (if no error)
- `window.webllmCDNLoading: false` (if CDN finished loading)

### Test 2: Check MLCEngine availability
```javascript
if (window.webllm && window.webllm.MLCEngine) {
  console.log('MLCEngine is available');
  console.log('typeof MLCEngine:', typeof window.webllm.MLCEngine);
} else {
  console.log('MLCEngine not found');
}
```

### Test 3: Check Network Requests
```javascript
// Open DevTools > Network tab, then run:
console.log('Check Network tab for:');
console.log('1. https://cdn.jsdelivr.net/npm/@mlc-ai/web-llm@0.2.32/dist/web-llm.js');
console.log('2. Should be status 200 (success)');
```

### Test 4: Manual CDN Test (if automatic fails)
```javascript
// Manually load from fallback
const script = document.createElement('script');
script.src = 'https://unpkg.com/@mlc-ai/web-llm@0.2.32/dist/web-llm.js';
script.onload = function() { console.log('unpkg load SUCCESS'); };
script.onerror = function() { console.log('unpkg load FAILED'); };
document.head.appendChild(script);
```

---

## Troubleshooting Flowchart

```
Open https://snapkittywest.github.io/rowm-polymorphic-notebook/index-app.html

Press F12 to open DevTools Console
Paste: console.log(typeof window.webllm)

├─ Shows "object"
│  └─ WebLLM loaded successfully ✓
│     Go to step: Load Model Test
│
├─ Shows "undefined"
│  ├─ Check Network tab for CDN request
│  │
│  ├─ Status 200 (loaded but not ready)
│  │  └─ Wait 5-10 seconds, refresh page
│  │     Paste: window.webllm ? 'Loaded' : 'Still failing'
│  │
│  ├─ Status 404 (not found)
│  │  └─ CDN URL is broken
│  │     Use fallback: https://unpkg.com/@mlc-ai/web-llm@0.2.32/dist/web-llm.js
│  │
│  ├─ Status CORS error
│  │  └─ Cross-Origin Request Blocked
│  │     Try from different browser or clear cookies
│  │
│  ├─ No request at all
│  │  └─ JavaScript isn't running
│  │     Check if index-app.html is accessible
│  │     Try hard refresh: Ctrl+Shift+R (Windows) or Cmd+Shift+R (Mac)
│  │
│  └─ Timeout / No response
│     └─ Network issue or CDN down
│        Try unpkg fallback (see Test 4 above)
```

---

## Load Model Test

Once WebLLM is confirmed loaded (`window.webllm !== undefined`):

```javascript
// Paste in console:
document.getElementById('ahmad-jit-load').click();
```

Watch the Ahmad Bot box:
- Button text changes to "LOADING" ✓
- System messages appear with progress
- Model downloads (takes 1-5 minutes)
- Status changes to "READY" when complete
- Send button enables

---

## What Was Fixed (July 27, 2026)

1. **Primary CDN** (jsDelivr) with proper onload/onerror handlers
2. **Fallback CDN** (unpkg) automatically triggered if primary fails
3. **Status tracking** variables: `window.webllmReady`, `window.webllmError`, `window.webllmCDNLoading`
4. **Error messages** in Ahmad Bot chat if CDN fails
5. **Console logging** to help diagnose issues
6. **Library check** in loadModel() - won't try if WebLLM not present
7. **Detailed error info** including CDN error reason

---

## CDNs Used

| CDN | URL | Status |
|-----|-----|--------|
| **Primary** | cdn.jsdelivr.net | Fast, reliable, cached |
| **Fallback** | unpkg.com | Always available, slower |

Both serve the exact same file: `@mlc-ai/web-llm@0.2.32/dist/web-llm.js`

---

## Files Changed

1. **index-app.html** — Added dual CDN loader with fallback
2. **js/ahmad-jit-ui.js** — Added WebLLM availability check before loading model
3. **js/ahmad-jit-engine.js** — Improved error messages with CDN status info

---

## Manual Testing Checklist

- [ ] Open page, check console: `typeof window.webllm` = "object"
- [ ] Check Network tab: CDN request returns 200
- [ ] Click "LOAD LOCAL MODEL" button
- [ ] See system messages appear
- [ ] Model download starts (progress visible in console)
- [ ] After 1-5 minutes, status changes to "READY"
- [ ] Try sending a message
- [ ] Response streams in real-time

---

## Still Having Issues?

1. **Check internet:** Open any website to confirm connection works
2. **Try different browser:** Chrome, Firefox, Safari, Edge
3. **Clear cache:** DevTools > Settings > Network > Uncheck "Disable cache"
4. **Try unpkg directly:** Manually paste Test 4 code above
5. **Check CSP:** Open DevTools, look for Content Security Policy errors
6. **Try private/incognito:** Rules out browser extensions
7. **Check proxy/firewall:** Corporate networks may block CDNs

---

**Report generated:** July 27, 2026
**Page:** https://snapkittywest.github.io/rowm-polymorphic-notebook/index-app.html
