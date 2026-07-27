/**
 * Ahmad JIT UI
 * Minimal chat interface for real WebLLM inference
 */

class AhmadJITUI {
    constructor() {
        this.elements = {};
        this.setupElements();
        this.setupEventListeners();
    }

    setupElements() {
        // Create launcher button if not exists
        if (!document.getElementById('ahmad-bot-launcher')) {
            const launcher = document.createElement('button');
            launcher.id = 'ahmad-bot-launcher';
            launcher.textContent = 'Ω AHMAD BOT';
            launcher.style.cssText = `
                position: fixed;
                bottom: 20px;
                right: 20px;
                z-index: 9999;
                padding: 12px 16px;
                background: #00d9ff;
                color: #0a0e27;
                border: 2px solid #0a0e27;
                border-radius: 4px;
                font-weight: bold;
                font-size: 14px;
                cursor: pointer;
                font-family: monospace;
            `;
            document.body.appendChild(launcher);
        }

        // Create panel if not exists
        if (!document.getElementById('ahmad-bot-panel')) {
            const panel = document.createElement('section');
            panel.id = 'ahmad-bot-panel';
            panel.hidden = true;
            panel.style.cssText = `
                position: fixed;
                bottom: 90px;
                right: 20px;
                width: 430px;
                max-height: 650px;
                background: #1a2654;
                border: 1px solid #2a3654;
                border-radius: 4px;
                z-index: 9998;
                display: flex;
                flex-direction: column;
                font-family: monospace;
                font-size: 12px;
                color: #e0e0e0;
            `;
            panel.innerHTML = `
                <header style="padding: 12px; border-bottom: 1px solid #2a3654; display: flex; justify-content: space-between; align-items: center;">
                    <strong>Ahmad Bot</strong>
                    <div>
                        <span id="ahmad-engine-status" style="margin-right: 12px; color: #a0a0a0;">OFFLINE</span>
                        <button id="ahmad-close" style="background: none; border: none; color: #e0e0e0; cursor: pointer; font-size: 16px;">×</button>
                    </div>
                </header>
                <div id="ahmad-load-view" style="padding: 12px;">
                    <button id="ahmad-load-model" style="width: 100%; padding: 8px; background: #00d9ff; color: #0a0e27; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; font-family: monospace;">LOAD LOCAL MODEL</button>
                    <progress id="ahmad-model-progress" max="1" value="0" style="width: 100%; margin-top: 12px;"></progress>
                    <div id="ahmad-progress-text" style="margin-top: 8px; color: #a0a0a0;"></div>
                </div>
                <div id="ahmad-messages" style="flex: 1; overflow-y: auto; padding: 12px; display: none;"></div>
                <form id="ahmad-form" style="display: none; padding: 12px; border-top: 1px solid #2a3654;">
                    <textarea id="ahmad-input" placeholder="Ask Ahmad Bot about this notebook" style="width: 100%; height: 60px; padding: 8px; background: #0a0e27; color: #e0e0e0; border: 1px solid #2a3654; border-radius: 4px; font-family: monospace; font-size: 12px; resize: none; display: none;" disabled></textarea>
                    <div style="display: flex; gap: 8px; margin-top: 8px;">
                        <button id="ahmad-send" type="submit" style="flex: 1; padding: 8px; background: #00d9ff; color: #0a0e27; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; font-family: monospace; display: none;" disabled>SEND</button>
                        <button id="ahmad-stop" type="button" style="flex: 1; padding: 8px; background: #ef4444; color: white; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; font-family: monospace; display: none;" disabled>STOP</button>
                    </div>
                </form>
            `;
            document.body.appendChild(panel);
        }

        this.elements.launcher = document.getElementById('ahmad-bot-launcher');
        this.elements.panel = document.getElementById('ahmad-bot-panel');
        this.elements.loadView = document.getElementById('ahmad-load-view');
        this.elements.loadBtn = document.getElementById('ahmad-load-model');
        this.elements.progress = document.getElementById('ahmad-model-progress');
        this.elements.progressText = document.getElementById('ahmad-progress-text');
        this.elements.messages = document.getElementById('ahmad-messages');
        this.elements.form = document.getElementById('ahmad-form');
        this.elements.input = document.getElementById('ahmad-input');
        this.elements.send = document.getElementById('ahmad-send');
        this.elements.stop = document.getElementById('ahmad-stop');
        this.elements.close = document.getElementById('ahmad-close');
        this.elements.status = document.getElementById('ahmad-engine-status');
    }

    setupEventListeners() {
        this.elements.launcher.addEventListener('click', () => this.togglePanel());
        this.elements.close.addEventListener('click', () => this.closePanel());
        this.elements.loadBtn.addEventListener('click', () => this.loadModel());
        this.elements.send.addEventListener('click', (e) => {
            e.preventDefault();
            this.sendMessage();
        });
        this.elements.stop.addEventListener('click', () => this.stopGeneration());
        this.elements.input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                this.sendMessage();
            }
        });
    }

    togglePanel() {
        if (this.elements.panel.hidden) {
            this.openPanel();
        } else {
            this.closePanel();
        }
    }

    openPanel() {
        this.elements.panel.hidden = false;
    }

    closePanel() {
        this.elements.panel.hidden = true;
    }

    async loadModel() {
        try {
            this.elements.loadBtn.disabled = true;
            this.updateStatus('LOADING');

            const engine = new AhmadJITEngine();
            window.ahmadEngine = engine;

            await engine.initialize('Qwen2-0.5B-Instruct-q4f32_1-MLC');

            this.updateStatus('READY');
            this.elements.loadView.style.display = 'none';
            this.elements.messages.style.display = 'block';
            this.elements.form.style.display = 'block';
            this.elements.input.style.display = 'block';
            this.elements.send.style.display = 'block';
            this.elements.input.disabled = false;
            this.elements.send.disabled = false;

            this.addMessage('system', '✓ Ahmad Bot ready. Ask about this notebook.');
        } catch (error) {
            this.updateStatus('ERROR');
            this.addMessage('system', `Error: ${error.message}`);
            this.elements.loadBtn.disabled = false;
        }
    }

    async sendMessage() {
        const text = this.elements.input.value.trim();
        if (!text || !window.ahmadEngine) return;

        this.addMessage('user', text);
        this.elements.input.value = '';
        this.elements.send.disabled = true;
        this.elements.stop.disabled = false;
        this.elements.stop.style.display = 'block';

        this.updateStatus('GENERATING');

        try {
            let response = '';
            await window.ahmadEngine.generate(text, (token) => {
                response += token;
                this.appendToken(token);
            });

            this.elements.stop.style.display = 'none';
        } catch (error) {
            this.addMessage('system', `Error: ${error.message}`);
            this.elements.stop.style.display = 'none';
        }

        this.updateStatus('READY');
        this.elements.send.disabled = false;
        this.elements.stop.disabled = true;
    }

    stopGeneration() {
        if (window.ahmadEngine) {
            window.ahmadEngine.stop();
        }
    }

    addMessage(role, text) {
        const msgDiv = document.createElement('div');
        msgDiv.style.cssText = `margin-bottom: 12px; padding: 8px; border-radius: 4px; ${
            role === 'user'
                ? 'background: #00d9ff; color: #0a0e27; text-align: right;'
                : role === 'system'
                ? 'background: #2a3654; color: #a0a0a0; font-style: italic;'
                : 'background: #0a0e27; color: #e0e0e0; border-left: 2px solid #00d9ff;'
        }`;
        msgDiv.textContent = text;
        this.elements.messages.appendChild(msgDiv);
        this.elements.messages.scrollTop = this.elements.messages.scrollHeight;
    }

    appendToken(token) {
        if (this.elements.messages.lastChild?.dataset?.role === 'assistant') {
            this.elements.messages.lastChild.textContent += token;
        } else {
            this.addMessage('assistant', token);
        }
        this.elements.messages.scrollTop = this.elements.messages.scrollHeight;
    }

    updateStatus(status) {
        this.elements.status.textContent = status;
        this.elements.status.style.color = status === 'READY' ? '#10b981' : '#a0a0a0';
    }
}

// Auto-initialize on load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.ahmadJITUI = new AhmadJITUI();
    });
} else {
    window.ahmadJITUI = new AhmadJITUI();
}
