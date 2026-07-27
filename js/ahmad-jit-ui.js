/**
 * Ahmad JIT UI
 * Centered chat box for real WebLLM inference
 */

class AhmadJITUI {
    constructor() {
        this.elements = {};
        this.messageQueue = [];
        this.setupElements();
        this.setupEventListeners();
    }

    setupElements() {
        // Create centered container
        if (!document.getElementById('ahmad-jit-box')) {
            const box = document.createElement('div');
            box.id = 'ahmad-jit-box';
            box.style.cssText = `
                position: fixed;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                width: 400px;
                height: 600px;
                background: #1a2654;
                border: 1px solid #2a3654;
                border-radius: 4px;
                z-index: 9999;
                display: flex;
                flex-direction: column;
                font-family: monospace;
                font-size: 12px;
                color: #e0e0e0;
                box-shadow: 0 0 30px rgba(0, 217, 255, 0.2);
            `;
            box.innerHTML = `
                <header style="padding: 16px; border-bottom: 1px solid #2a3654; display: flex; justify-content: space-between; align-items: center;">
                    <strong style="font-size: 14px;">Ω Ahmad JIT Assistant</strong>
                    <span id="ahmad-jit-status" style="color: #a0a0a0; font-size: 11px;">OFFLINE</span>
                </header>
                <div id="ahmad-jit-messages" style="flex: 1; overflow-y: auto; padding: 12px; display: flex; flex-direction: column; gap: 8px;"></div>
                <div id="ahmad-jit-composer" style="padding: 12px; border-top: 1px solid #2a3654; display: flex; flex-direction: column; gap: 8px;">
                    <textarea id="ahmad-jit-input" placeholder="Ask about this notebook..." style="width: 100%; height: 50px; padding: 8px; background: #0a0e27; color: #e0e0e0; border: 1px solid #2a3654; border-radius: 4px; font-family: monospace; font-size: 12px; resize: none;" disabled></textarea>
                    <div style="display: flex; gap: 8px;">
                        <button id="ahmad-jit-load" style="flex: 1; padding: 8px; background: #00d9ff; color: #0a0e27; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; font-family: monospace; font-size: 12px;">LOAD LOCAL MODEL</button>
                    </div>
                    <div id="ahmad-jit-action-buttons" style="display: none; gap: 8px;">
                        <button id="ahmad-jit-send" style="flex: 1; padding: 8px; background: #00d9ff; color: #0a0e27; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; font-family: monospace; font-size: 12px;" disabled>SEND</button>
                        <button id="ahmad-jit-stop" style="flex: 1; padding: 8px; background: #ef4444; color: white; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; font-family: monospace; font-size: 12px;" disabled>STOP</button>
                    </div>
                </div>
            `;
            document.body.appendChild(box);
        }

        this.elements.box = document.getElementById('ahmad-jit-box');
        this.elements.messages = document.getElementById('ahmad-jit-messages');
        this.elements.input = document.getElementById('ahmad-jit-input');
        this.elements.loadBtn = document.getElementById('ahmad-jit-load');
        this.elements.send = document.getElementById('ahmad-jit-send');
        this.elements.stop = document.getElementById('ahmad-jit-stop');
        this.elements.status = document.getElementById('ahmad-jit-status');
        this.elements.composer = document.getElementById('ahmad-jit-composer');
        this.elements.actionButtons = document.getElementById('ahmad-jit-action-buttons');
    }

    setupEventListeners() {
        this.elements.loadBtn.addEventListener('click', () => this.loadModel());
        this.elements.send.addEventListener('click', () => this.sendMessage());
        this.elements.stop.addEventListener('click', () => this.stopGeneration());
        this.elements.input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                this.sendMessage();
            }
        });
    }

    async loadModel() {
        try {
            // Check if WebLLM library is loaded
            if (!window.webllm) {
                // Check if it's still loading
                if (window.webllmCDNLoading) {
                    this.addMessage('system', 'WebLLM library is still loading from CDN. Please wait a moment and try again.');
                    this.elements.loadBtn.disabled = false;
                    return;
                }

                // Check for specific error
                if (window.webllmError) {
                    throw new Error(`WebLLM failed to load: ${window.webllmError}. Check your internet connection and try again.`);
                }

                throw new Error('WebLLM library not available. Ensure index-app.html includes WebLLM CDN script.');
            }

            this.elements.loadBtn.disabled = true;
            this.updateStatus('LOADING');
            this.addMessage('system', 'Initializing WebLLM engine and downloading model (500MB+)...');
            this.addMessage('system', 'This may take 1-5 minutes on first load. Check browser console for progress.');

            // Verify engine exists (from ahmad-jit-engine.js)
            if (typeof AhmadJITEngine === 'undefined') {
                throw new Error('AhmadJITEngine not loaded. Check that js/ahmad-jit-engine.js is included.');
            }

            const engine = new AhmadJITEngine();
            window.ahmadEngine = engine;

            // Load the model - this is a long operation
            try {
                await engine.initialize('Qwen2-0.5B-Instruct-q4f32_1-MLC');
            } catch (initError) {
                throw new Error(`Model initialization failed: ${initError.message}`);
            }

            this.updateStatus('READY');
            this.elements.loadBtn.style.display = 'none';
            this.elements.actionButtons.style.display = 'flex';
            this.elements.input.disabled = false;
            this.elements.send.disabled = false;

            this.addMessage('system', '✓ Ahmad Bot ready. Ask about this notebook.');
        } catch (error) {
            this.updateStatus('ERROR');
            this.addMessage('system', `Error: ${error.message}`);
            this.addMessage('system', 'Check browser console (F12) for detailed error logs.');
            this.elements.loadBtn.disabled = false;
            console.error('Model load failed:', error);
        }
    }

    async sendMessage() {
        const text = this.elements.input.value.trim();
        if (!text) {
            this.addMessage('system', 'Please enter a message.');
            return;
        }

        if (!window.ahmadEngine) {
            this.addMessage('system', 'Error: Engine not initialized. Load model first.');
            return;
        }

        this.addMessage('user', text);
        this.elements.input.value = '';
        this.elements.input.disabled = true;
        this.elements.send.disabled = true;
        this.elements.stop.disabled = false;
        this.elements.loadBtn.disabled = true;

        this.updateStatus('GENERATING');

        try {
            let response = '';
            await window.ahmadEngine.generate(text, (token) => {
                response += token;
                this.appendToken(token);
            });

            if (!response || response.length === 0) {
                this.addMessage('system', 'Warning: No response generated.');
            }
        } catch (error) {
            const errorMsg = error instanceof Error ? error.message : String(error);
            this.addMessage('system', `Generation error: ${errorMsg}`);
            console.error('Message generation failed:', error);
        }

        this.updateStatus('READY');
        this.elements.input.disabled = false;
        this.elements.send.disabled = false;
        this.elements.stop.disabled = true;
        this.elements.loadBtn.disabled = false;
        this.elements.input.focus();
    }

    stopGeneration() {
        if (window.ahmadEngine) {
            window.ahmadEngine.stop();
        }
    }

    addMessage(role, text) {
        const msgDiv = document.createElement('div');
        const bgColor = role === 'user' ? '#00d9ff' : role === 'system' ? '#2a3654' : '#0a0e27';
        const textColor = role === 'user' ? '#0a0e27' : role === 'system' ? '#a0a0a0' : '#e0e0e0';
        const border = role === 'assistant' ? '1px solid #00d9ff' : 'none';

        msgDiv.style.cssText = `
            padding: 8px;
            border-radius: 4px;
            background: ${bgColor};
            color: ${textColor};
            word-wrap: break-word;
            white-space: pre-wrap;
            border: ${border};
            font-style: ${role === 'system' ? 'italic' : 'normal'};
        `;
        msgDiv.dataset.role = role;
        msgDiv.textContent = text;
        this.elements.messages.appendChild(msgDiv);
        this.elements.messages.scrollTop = this.elements.messages.scrollHeight;
    }

    appendToken(token) {
        if (!token || typeof token !== 'string') {
            return; // Silently skip invalid tokens
        }

        const messages = this.elements.messages;
        if (!messages) {
            console.error('Messages container not found');
            return;
        }

        if (messages.lastChild && messages.lastChild.dataset && messages.lastChild.dataset.role === 'assistant') {
            // Append to existing assistant message (textContent is XSS-safe)
            messages.lastChild.textContent += token;
        } else {
            // Create new assistant message
            this.addMessage('assistant', token);
        }

        // Auto-scroll to bottom
        try {
            messages.scrollTop = messages.scrollHeight;
        } catch (e) {
            console.warn('Scroll failed:', e);
        }
    }

    updateStatus(status) {
        this.elements.status.textContent = status;
        this.elements.status.style.color =
            status === 'READY' ? '#10b981' :
            status === 'ERROR' ? '#ef4444' :
            status === 'GENERATING' ? '#fbbf24' :
            '#a0a0a0';
    }
}

// Auto-initialize on load (with WebLLM readiness check)
function initializeAhmadUI() {
    try {
        if (typeof AhmadJITUI !== 'undefined') {
            window.ahmadJITUI = new AhmadJITUI();
            console.log('Ahmad JIT UI initialized successfully');

            // Check WebLLM status and log it
            if (window.webllm) {
                console.log('WebLLM library detected:', typeof window.webllm);
            } else if (window.webllmCDNLoading) {
                console.warn('WebLLM still loading from CDN...');
            } else if (window.webllmError) {
                console.error('WebLLM load error:', window.webllmError);
            } else {
                console.warn('WebLLM not loaded and no error recorded. CDN may have failed silently.');
            }
        } else {
            console.error('AhmadJITUI class not found');
        }
    } catch (error) {
        console.error('Failed to initialize Ahmad JIT UI:', error);
    }
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeAhmadUI);
} else {
    initializeAhmadUI();
}

// Provide global hook for manual initialization if needed
window.initAhmadUI = initializeAhmadUI;

// Also monitor WebLLM loading status
console.log('Page ready. WebLLM CDN is loading...');
