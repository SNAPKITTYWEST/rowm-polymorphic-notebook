/**
 * Ahmad Bot UI
 * Chat interface, launcher button, and panel management
 * Mounts to existing #jit-launcher and #jit-panel DOM elements
 */

class AhmadBotUI {
    constructor() {
        this.engine = null;
        this.elements = this.initElements();
        this.state = {
            isOpen: false,
            isMinimized: false,
            hasModel: false,
            isDragging: false,
            panelStartX: 0,
            panelStartY: 0,
        };

        this.setupEventListeners();
        this.ensureStyles();
        this.updateStatus('OFFLINE');
    }

    /**
     * Initialize DOM elements
     */
    initElements() {
        return {
            launcher: document.getElementById('jit-launcher'),
            launcherButton: document.getElementById('jit-toggle'),
            progressRing: document.getElementById('jit-progress-ring'),
            panel: document.getElementById('jit-panel'),
            header: document.querySelector('.jit-header'),
            messagesContainer: document.getElementById('jit-messages'),
            statusBadge: document.getElementById('jit-status'),
            input: document.getElementById('jit-input'),
            sendBtn: document.getElementById('jit-send'),
            stopBtn: document.getElementById('jit-stop'),
            clearBtn: document.getElementById('jit-clear'),
            minimizeBtn: document.getElementById('jit-minimize'),
            closeBtn: document.getElementById('jit-close'),
        };
    }

    /**
     * Setup all event listeners
     */
    setupEventListeners() {
        // Launcher button
        this.elements.launcherButton.addEventListener('click', () => this.togglePanel());

        // Panel controls
        this.elements.minimizeBtn.addEventListener('click', () => this.toggleMinimize());
        this.elements.closeBtn.addEventListener('click', () => this.closePanel());
        this.elements.clearBtn.addEventListener('click', () => this.clearMessages());

        // Chat input
        this.elements.input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey) {
                e.preventDefault();
                this.sendMessage();
            }
        });

        this.elements.sendBtn.addEventListener('click', () => this.sendMessage());
        this.elements.stopBtn.addEventListener('click', () => this.stopGeneration());

        // Panel dragging
        this.elements.header.addEventListener('mousedown', (e) => this.startDrag(e));
        document.addEventListener('mousemove', (e) => this.drag(e));
        document.addEventListener('mouseup', () => this.endDrag());

        // Auto-focus input when panel opens
        this.elements.panel.addEventListener('transitionend', () => {
            if (!this.state.isOpen) return;
            if (this.state.hasModel) {
                this.elements.input.focus();
            }
        });
    }

    /**
     * Toggle panel visibility
     */
    async togglePanel() {
        if (this.state.isOpen) {
            this.closePanel();
        } else {
            this.openPanel();
        }
    }

    /**
     * Open panel
     */
    async openPanel() {
        this.state.isOpen = true;
        this.elements.panel.classList.remove('hidden');
        this.elements.launcherButton.classList.add('active');

        // Initialize engine on first open
        if (!this.engine) {
            await this.initializeEngine();
        }

        // Wait for animation
        await new Promise(r => setTimeout(r, 100));
        this.elements.input.focus();
    }

    /**
     * Close panel
     */
    closePanel() {
        this.state.isOpen = false;
        this.elements.panel.classList.add('hidden');
        this.elements.launcherButton.classList.remove('active');
    }

    /**
     * Toggle minimize
     */
    toggleMinimize() {
        this.state.isMinimized = !this.state.isMinimized;

        if (this.state.isMinimized) {
            this.elements.panel.classList.add('minimized');
        } else {
            this.elements.panel.classList.remove('minimized');
        }
    }

    /**
     * Initialize WebLLM engine
     */
    async initializeEngine() {
        try {
            if (!AhmadWebLLMEngine.hasWebLLM()) {
                this.updateStatus('ERROR');
                this.addMessage({
                    role: 'system',
                    content: '✗ WebLLM not available. Ensure @mlc-ai/web-llm is loaded.',
                });
                return;
            }

            this.updateStatus('LOADING');
            this.elements.launcherButton.classList.add('loading');
            this.addMessage({
                role: 'system',
                content: '⏳ Initializing Ahmad Bot...',
            });

            // Create engine
            this.engine = new AhmadWebLLMEngine({
                temperature: 0.3,
                topP: 0.85,
                maxTokens: 512,
            });

            // Setup listeners
            this.engine.on('statusChanged', (status) => this.updateStatus(status));
            this.engine.on('token', (token) => this.appendToken(token));
            this.engine.on('generationStart', () => {
                this.elements.sendBtn.disabled = true;
                this.elements.stopBtn.disabled = false;
            });
            this.engine.on('generationComplete', () => {
                this.elements.sendBtn.disabled = false;
                this.elements.stopBtn.disabled = true;
            });
            this.engine.on('generationStopped', () => {
                this.elements.sendBtn.disabled = false;
                this.elements.stopBtn.disabled = true;
            });
            this.engine.on('error', (error) => this.onError(error));

            // Initialize
            await this.engine.initialize();

            this.state.hasModel = true;
            this.elements.launcherButton.classList.remove('loading');
            this.elements.input.disabled = false;
            this.elements.sendBtn.disabled = false;

            const webgpu = AhmadWebLLMEngine.hasWebGPU() ? 'WebGPU' : 'CPU';
            this.addMessage({
                role: 'system',
                content: `✓ Ready. Model: ${this.engine.modelId} | Acceleration: ${webgpu}`,
            });

        } catch (error) {
            this.updateStatus('ERROR');
            this.elements.launcherButton.classList.remove('loading');
            this.addMessage({
                role: 'system',
                content: `✗ Failed: ${error.message}`,
            });
            console.error('[Ahmad Bot UI] Init error:', error);
        }
    }

    /**
     * Send user message
     */
    async sendMessage() {
        const userMessage = this.elements.input.value.trim();

        if (!userMessage || !this.engine) {
            return;
        }

        if (this.engine.isGenerating) {
            this.addMessage({
                role: 'system',
                content: 'Generation in progress. Click Stop or wait.',
            });
            return;
        }

        // Add user message to UI
        this.addMessage({
            role: 'user',
            content: userMessage,
        });

        // Clear input
        this.elements.input.value = '';
        this.elements.input.style.height = 'auto';

        // Build system prompt with notebook context
        const systemPrompt = this.engine.buildSystemPrompt(userMessage);

        // Generate response
        try {
            await this.engine.generate(userMessage, systemPrompt);
        } catch (error) {
            if (error.message !== 'Generation aborted') {
                this.onError(error.message);
            }
        }
    }

    /**
     * Append token to current message
     */
    appendToken(token) {
        const messages = this.elements.messagesContainer.querySelectorAll('.jit-message');
        const lastMessage = messages[messages.length - 1];

        if (lastMessage && lastMessage.dataset.role === 'assistant') {
            const bubble = lastMessage.querySelector('.jit-message-bubble');
            bubble.textContent += token;

            // Auto-scroll to bottom
            if (!this.isUserScrolledUp()) {
                this.elements.messagesContainer.scrollTop = this.elements.messagesContainer.scrollHeight;
            }
        }
    }

    /**
     * Check if user scrolled up
     */
    isUserScrolledUp() {
        const c = this.elements.messagesContainer;
        return c.scrollTop < c.scrollHeight - c.clientHeight - 100;
    }

    /**
     * Add message to UI
     */
    addMessage({ role, content, streaming = false }) {
        const messageDiv = document.createElement('div');
        messageDiv.className = `jit-message jit-message-${role}`;
        messageDiv.dataset.role = role;

        const bubble = document.createElement('div');
        bubble.className = 'jit-message-bubble';

        // Set content
        if (role === 'system') {
            bubble.style.fontSize = '11px';
            bubble.style.opacity = '0.8';
            bubble.style.fontStyle = 'italic';
        }

        bubble.textContent = content;
        messageDiv.appendChild(bubble);
        this.elements.messagesContainer.appendChild(messageDiv);

        // Auto-scroll
        this.elements.messagesContainer.scrollTop = this.elements.messagesContainer.scrollHeight;

        return messageDiv;
    }

    /**
     * Clear messages
     */
    clearMessages() {
        this.elements.messagesContainer.innerHTML = '';
        if (this.engine) {
            this.engine.clearHistory();
        }
        this.addMessage({
            role: 'system',
            content: 'Chat history cleared.',
        });
    }

    /**
     * Stop generation
     */
    stopGeneration() {
        if (this.engine && this.engine.isGenerating) {
            this.engine.interrupt();
        }
    }

    /**
     * Update status badge
     */
    updateStatus(status) {
        const statusMap = {
            'OFFLINE': 'offline',
            'LOADING': 'loading',
            'READY': 'ready',
            'GENERATING': 'generating',
            'ERROR': 'error',
        };

        const cssClass = statusMap[status] || 'offline';

        // Remove old status classes
        this.elements.statusBadge.className = 'jit-status-badge';
        this.elements.statusBadge.classList.add(`jit-status-${cssClass}`);
        this.elements.statusBadge.textContent = status;
    }

    /**
     * Handle errors
     */
    onError(errorMessage) {
        this.addMessage({
            role: 'system',
            content: `✗ Error: ${errorMessage}`,
        });
        this.updateStatus('ERROR');
    }

    /**
     * Panel dragging
     */
    startDrag(e) {
        if (e.target !== this.elements.header && !this.elements.header.contains(e.target)) {
            return;
        }
        this.state.isDragging = true;
        this.state.panelStartX = e.clientX - this.elements.panel.offsetLeft;
        this.state.panelStartY = e.clientY - this.elements.panel.offsetTop;
        this.elements.panel.style.cursor = 'grabbing';
    }

    drag(e) {
        if (!this.state.isDragging) return;

        const x = e.clientX - this.state.panelStartX;
        const y = e.clientY - this.state.panelStartY;

        this.elements.panel.style.left = Math.max(0, Math.min(x, window.innerWidth - this.elements.panel.offsetWidth)) + 'px';
        this.elements.panel.style.top = Math.max(0, Math.min(y, window.innerHeight - this.elements.panel.offsetHeight)) + 'px';
    }

    endDrag() {
        this.state.isDragging = false;
        this.elements.panel.style.cursor = 'auto';
    }

    /**
     * Ensure styles are present
     */
    ensureStyles() {
        if (document.querySelector('style[data-ahmad-bot-styles]')) {
            return;
        }

        const style = document.createElement('style');
        style.setAttribute('data-ahmad-bot-styles', 'true');
        style.textContent = `
            .jit-status-badge {
                display: inline-block;
                font-size: 10px;
                font-weight: 700;
                text-transform: uppercase;
                padding: 3px 8px;
                border-radius: 3px;
                letter-spacing: 0.5px;
            }

            .jit-status-offline {
                background: rgba(107, 114, 128, 0.3);
                color: #a0a0a0;
            }

            .jit-status-loading {
                background: rgba(59, 130, 246, 0.3);
                color: #60a5fa;
                animation: ahmad-pulse 1.5s ease-in-out infinite;
            }

            .jit-status-ready {
                background: rgba(16, 185, 129, 0.3);
                color: #10b981;
            }

            .jit-status-generating {
                background: rgba(0, 217, 255, 0.3);
                color: #00d9ff;
                animation: ahmad-pulse 1s ease-in-out infinite;
            }

            .jit-status-error {
                background: rgba(239, 68, 68, 0.3);
                color: #ef4444;
            }

            @keyframes ahmad-pulse {
                0%, 100% { opacity: 0.7; }
                50% { opacity: 1; }
            }

            .jit-message-user .jit-message-bubble {
                background: #00d9ff;
                color: #0a0e27;
            }

            .jit-message-assistant .jit-message-bubble {
                background: #1a1a2e;
                color: #e0e0e0;
            }
        `;
        document.head.appendChild(style);
    }

    /**
     * Get UI stats
     */
    getStats() {
        return {
            isOpen: this.state.isOpen,
            hasModel: this.state.hasModel,
            status: this.engine?.status || 'OFFLINE',
            messageCount: this.elements.messagesContainer.querySelectorAll('.jit-message').length,
        };
    }
}

// Initialize on DOM ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.ahmadBotUI = new AhmadBotUI();
    });
} else {
    window.ahmadBotUI = new AhmadBotUI();
}

// Export
if (typeof module !== 'undefined' && module.exports) {
    module.exports = AhmadBotUI;
}
