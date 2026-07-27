# WebLLM CDN Diagnostics & Troubleshooting

## Issue: "WebLLM library is still loading from CDN. Please wait..."

This message appears when the CDN takes longer than expected to load. This guide helps you troubleshoot.

---

## Quick Diagnostic (Copy-Paste to Console)

Open https://snapkittywest.github.io/rowm-polymorphic-notebook/index-app.html, press F12, then paste:

```javascript
// Check CDN loading status
console.log('=== WebLLM CDN Status ===');
console.log('Ready:', window.webllmReady);
console.log('Loading:', window.webllmCDNLoading);
console.log('Error:', window.webllmError);
console.log('Library type:', typeof window.webllm);
console.log('Load time (ms):', window.webllmLoadTimestamp ? Date.now() - window.webllmLoadTimestamp : 'N/A');

// If loaded, show MLCEngine
if (window.webllm && window.webllm.MLCEngine) {
    console.log('MLCEngine available:', typeof window.webllm.MLCEngine);
} else if (window.webllm) {
    console.log('WebLLM loaded but MLCEngine not found');
} else {
    console.log('WebLLM not loaded yet');
}
```

**Expected Output (Success):**
```
Ready: true
Loading: false
Error: null
Library type: object
Load time: 2500 (milliseconds)
MLCEngine available: function
```

**Expected Output (Slow Loading):**
```
Ready: false
Loading: true
Error: null
Library type: undefined
Load time: 8500
MLCEngine available: N/A
```

---

## Network Tab Analysis

1. Open DevTools > Network tab
2. Reload page (F5)
3. Look for: `web-llm.js` request

**Check the following:**

| Item | Good | Bad | Fix |
|------|------|-----|-----|
| **Status** | 200, 304 | 404, 403, 0 | Check CDN URL or internet |
| **Time** | <5s | >10s | CDN slow, try unpkg fallback |
| **Size** | 1.5MB | Much larger | File corrupted, retry |
| **CORS** | No error | CORS error | Network/firewall blocking |

**CDN URLs Used (in order):**
1. Primary: `https://cdn.jsdelivr.net/npm/@mlc-ai/web-llm@0.2.32/dist/web-llm.js`
2. Fallback: `https://unpkg.com/@mlc-ai/web-llm@0.2.32/dist/web-llm.js`

---

## Common Issues & Solutions

### Issue 1: CDN Loads But Takes 10+ Seconds

**Symptoms:**
- Console shows: `Ready: false`, `Loading: true` for >10 seconds
- Network tab shows: Status 200 but very slow transfer

**Causes:**
- Slow internet connection
- CDN server responding slowly
- Large file size (1.5MB)

**Solutions:**
```javascript
// Option A: Force retry from backup CDN
window.retryWebLLMLoad();

// Option B: Wait longer (CDN may still load)
// Just wait 30 more seconds, then try LOAD LOCAL MODEL again
```

### Issue 2: CDN Request Times Out (No Response)

**Symptoms:**
- Console shows: `Error: jsDelivr timeout`
- Network tab shows: CDN request pending indefinitely or canceled

**Causes:**
- Network timeout (CDN unreachable)
- Firewall/proxy blocking CDN
- ISP throttling or filtering

**Solutions:**
```javascript
// Automatic: Fallback to unpkg triggers after 8 seconds
// Manual retry:
window.retryWebLLMLoad();

// If still fails, try from corporate WiFi or personal hotspot
```

### Issue 3: CORS Error

**Symptoms:**
- Console shows: `Access to XMLHttpRequest from 'file://...' has been blocked by CORS policy`
- Or: `Access to fetch from 'https://cdn.jsdelivr.net' blocked by CORS`

**Causes:**
- Opening file:// directly (not via HTTP server)
- Corporate proxy filtering cross-origin requests

**Solutions:**
```javascript
// Must use HTTP/HTTPS, not file://
// Visit: https://snapkittywest.github.io/rowm-polymorphic-notebook/index-app.html
// NOT: file:///C:/Users/.../index-app.html

// If corporate network:
// 1. Ask IT to whitelist: cdn.jsdelivr.net
// 2. Try from personal WiFi/hotspot
// 3. Use VPN if allowed
```

### Issue 4: WebLLM Loads But window.webllm Is undefined

**Symptoms:**
- Network tab shows: 200 (file downloaded)
- Console shows: `typeof window.webllm: undefined`

**Causes:**
- Script loaded but didn't execute properly
- File corrupted or wrong version
- Browser doesn't support WASM or async

**Solutions:**
```javascript
// Manually retry:
window.retryWebLLMLoad();

// Check browser compatibility:
console.log('WebAssembly:', typeof WebAssembly !== 'undefined');
console.log('Promise:', typeof Promise !== 'undefined');
console.log('Symbol.asyncIterator:', Symbol.asyncIterator);
```

### Issue 5: "Please Wait..." Message Never Disappears

**Symptoms:**
- Message: "WebLLM library is still loading from CDN. Please wait..."
- Appears indefinitely, button doesn't re-enable

**Causes:**
- CDN truly stuck (not responding)
- Browser timeout setting too high
- JavaScript error preventing completion

**Solutions:**
```javascript
// Try manual retry:
window.retryWebLLMLoad();

// Check for JavaScript errors:
// Press F12, look for red errors in Console

// Force manual timeout:
setTimeout(function() {
    console.log('Manual timeout fired');
    if (!window.webllm) {
        console.log('CDN still not loaded after 30 seconds, giving up');
        window.webllmError = 'Timeout after 30 seconds';
        window.webllmCDNLoading = false;
        // Click LOAD LOCAL MODEL button to show error
    }
}, 30000);
```

---

## Manual Retry Steps

**When to use:** If "Please wait..." message persists for >15 seconds

1. Open Console (F12)
2. Paste: `window.retryWebLLMLoad()`
3. Wait 5-10 seconds
4. Check console for: "WebLLM loaded from unpkg"
5. Try clicking "LOAD LOCAL MODEL" button again

**Expected result:**
- Console shows: `WebLLM loaded from unpkg X milliseconds`
- Status in Ahmad Bot changes from OFFLINE to READY

---

## Browser Console Command Reference

| Command | Purpose | Expected Output |
|---------|---------|-----------------|
| `window.webllmReady` | Check if loaded | `true` |
| `typeof window.webllm` | Check library exists | `object` |
| `window.webllmError` | Show error reason | `null` or error text |
| `Date.now() - window.webllmLoadTimestamp` | Check load time (ms) | `2500` (normal) or `>10000` (slow) |
| `window.retryWebLLMLoad()` | Manual retry | Loads from unpkg |
| `window.initAhmadUI()` | Re-initialize UI | No visible change |
| `window.ahmadJITUI.updateStatus('READY')` | Force status change | Status badge changes |

---

## Network Conditions to Test

These represent different scenarios:

```javascript
// Test 1: Slow CDN (simulates poor connection)
// Just wait longer (15+ seconds expected)

// Test 2: Timeout scenario
// CDN doesn't respond after 8 seconds → Fallback to unpkg

// Test 3: Corporate network
// May be blocked by proxy → Try personal WiFi

// Test 4: Mobile hotspot
// May have data limits → Download may be slow (5-10 min)
```

---

## Expected CDN Load Times

| Connection | Expected Time | Timeout |
|------------|---------------|---------|
| Fiber (100+ Mbps) | 2-3 seconds | 8 seconds |
| Broadband (10-50 Mbps) | 3-5 seconds | 8 seconds |
| DSL (3-10 Mbps) | 5-10 seconds | 8 seconds |
| Mobile 4G (5-20 Mbps) | 5-15 seconds | 8 seconds |
| Mobile 3G (<5 Mbps) | 15-30 seconds | 8 seconds (fallback) |
| Satellite | 30+ seconds | 8 seconds (fallback) |

**Note:** If CDN takes >8 seconds, automatic fallback to unpkg triggers.

---

## Full Diagnostic Report (Copy-Paste Everything)

Paste this entire script in console to get a complete report:

```javascript
console.log('=== WebLLM CDN FULL DIAGNOSTIC REPORT ===');
console.log('Timestamp:', new Date().toISOString());
console.log('');
console.log('=== Library Status ===');
console.log('window.webllm:', typeof window.webllm);
console.log('window.webllmReady:', window.webllmReady);
console.log('window.webllmCDNLoading:', window.webllmCDNLoading);
console.log('window.webllmError:', window.webllmError);
console.log('');
console.log('=== Performance ===');
console.log('Load time (ms):', window.webllmLoadTimestamp ? Date.now() - window.webllmLoadTimestamp : 'Not started');
console.log('');
console.log('=== MLCEngine ===');
if (window.webllm) {
    console.log('MLCEngine available:', typeof window.webllm.MLCEngine);
    console.log('MLCEngine is function:', typeof window.webllm.MLCEngine === 'function');
} else {
    console.log('WebLLM not loaded, cannot check MLCEngine');
}
console.log('');
console.log('=== Browser Support ===');
console.log('WebAssembly:', typeof WebAssembly);
console.log('Promise:', typeof Promise);
console.log('Async/Await:', typeof (async function(){}).constructor);
console.log('');
console.log('=== Recommendations ===');
if (window.webllmReady) {
    console.log('✓ WebLLM is ready. Try clicking LOAD LOCAL MODEL.');
} else if (window.webllmCDNLoading) {
    console.log('⟳ WebLLM still loading. Wait a moment and try again.');
    console.log('  OR manually retry: window.retryWebLLMLoad()');
} else if (window.webllmError) {
    console.log('✗ WebLLM failed to load. Error:', window.webllmError);
    console.log('  Trying manual retry: window.retryWebLLMLoad()');
} else {
    console.log('? WebLLM status unknown. Try refreshing the page.');
}
```

---

## Still Having Issues?

Try these in order:

1. **Hard Refresh** — Ctrl+Shift+R (Windows) / Cmd+Shift+R (Mac)
2. **Clear Cache** — DevTools > Settings > Network > Disable cache (checked)
3. **Manual Retry** — Paste: `window.retryWebLLMLoad()`
4. **Different Browser** — Try Chrome, Firefox, Safari, Edge
5. **Different Network** — Try phone hotspot or different WiFi
6. **Check Internet** — Open google.com, YouTube, etc. to confirm connection works

---

## For Developers

**Key Variables:**
- `window.webllmReady` — Boolean, true when library loaded
- `window.webllmError` — String, error reason if failed
- `window.webllmCDNLoading` — Boolean, true while loading
- `window.webllmLoadTimestamp` — Number, when loading started (Date.now())
- `window.webllmCallbacks` — Array, functions to call when ready

**Key Functions:**
- `window.retryWebLLMLoad()` — Manual fallback to unpkg
- `window.onWebLLMReady(callback)` — Register callback for when ready
- `window.fireWebLLMCallbacks()` — Internal, fires all callbacks

**CDN Load Timeout:** 8 seconds (after which fallback triggers)

---

**Last Updated:** July 27, 2026  
**Page:** https://snapkittywest.github.io/rowm-polymorphic-notebook/index-app.html
