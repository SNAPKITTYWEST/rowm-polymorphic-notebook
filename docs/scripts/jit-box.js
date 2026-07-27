/**
 * JIT Box — Floating Chat Interface
 * Embedded WebLLM assistant for notebook interaction
 */

class JITBox {
    constructor() {
        this.elements = this.initElements();
        this.engine = null;
        this.contextBuilder = null;
        this.unicodeEngine = null;
        this.receiptChain = null;

        this.state = {
            isOpen: false,
            isMinimized: false,
            hasModel: false,
        };

        this.setupEventListeners();
        this.injectDefaultStyling();
    }

    /**
     * Initialize DOM elements
     */
    initElements() {
        return {
            launcher: document.getElementById('jit-launcher'),
            button: document.getElementById('jit-toggle'),
            panel: document.getElementById('jit-panel'),
            messagesContainer: document.getElementById('jit-messages'),
            input: document.getElementById('jit-input'),
            sendBtn: document.getElementById('jit-send'),
            stopBtn: document.getElementById('jit-stop'),
            clearBtn: document.getElementById('jit-clear'),
            minimizeBtn: document.getElementById('jit-minimize'),
            closeBtn: document.getElementById('jit-close'),
            statusBadge: document.getElementById('jit-status'),
            header: document.querySelector('.jit-header'),
        };
    }

    /**
     * Setup event listeners
     */
    setupEventListeners() {
        // Launcher
        this.elements.button.addEventListener('click', () => this.togglePanel());

        // Panel controls
        this.elements.minimizeBtn.addEventListener('click', () => this.toggleMinimize());
        this.elements.closeBtn.addEventListener('click', () => this.closePanel());
        this.elements.clearBtn.addEventListener('click', () => this.clearMessages());

        // Chat input
        this.elements.input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                this.sendMessage();
            }
        });

        this.elements.sendBtn.addEventListener('click', () => this.sendMessage());
        this.elements.stopBtn.addEventListener('click', () => this.stopGeneration());
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
     * Open panel and initialize engine
     */
    async openPanel() {
        this.state.isOpen = true;
        this.elements.panel.classList.remove('hidden');
        this.elements.button.classList.add('active');

        // Initialize engine on first open
        if (!this.engine) {
            await this.initializeEngine();
        }

        this.elements.input.focus();
    }

    /**
     * Close panel
     */
    closePanel() {
        this.state.isOpen = false;
        this.elements.panel.classList.add('hidden');
        this.elements.button.classList.remove('active');
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
            // Check WebLLM availability
            if (!WebLLMEngine.hasWebLLM()) {
                this.updateStatus('ERROR');
                this.addMessage({
                    role: 'system',
                    content: 'WebLLM not available. Load @mlc-ai/web-llm script first.',
                });
                return;
            }

            this.updateStatus('LOADING');
            this.elements.button.classList.add('loading');

            // Initialize engines
            this.engine = new WebLLMEngine({
                temperature: 0.2,
                topP: 0.9,
                maxTokens: 256,
            });

            this.contextBuilder = new NotebookContextBuilder();
            this.unicodeEngine = new UnicodeIREngine();
            this.receiptChain = new WORMReceiptChain();

            // Listen to engine events
            this.engine.on('statusChanged', (status) => this.updateStatus(status));
            this.engine.on('token', (token) => this.appendToken(token));
            this.engine.on('generationComplete', (response) => this.onGenerationComplete(response));
            this.engine.on('error', (error) => this.onGenerationError(error));

            // Initialize engine
            await this.engine.initialize();

            this.state.hasModel = true;
            this.elements.button.classList.remove('loading');
            this.updateStatus('READY');
            this.elements.input.disabled = false;
            this.elements.sendBtn.disabled = false;

            this.addMessage({
                role: 'system',
                content: `✓ Ready. Using ${this.engine.model}. WebGPU: ${WebLLMEngine.hasWebGPU() ? 'Yes' : 'CPU mode'}`,
            });
        } catch (error) {
            this.updateStatus('ERROR');
            this.elements.button.classList.remove('loading');
            this.addMessage({
                role: 'system',
                content: `✗ Engine initialization failed: ${error.message}`,
            });
        }
    }

    /**
     * Send message
     */
    async sendMessage() {
        const userMessage = this.elements.input.value.trim();

        if (!userMessage || !this.engine || this.engine.isGenerating) {
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

        // Build context
        let systemPrompt = '';
        try {
            const packet = this.contextBuilder.buildContextPacket(userMessage);
            systemPrompt = this.contextBuilder.formatSystemPrompt(packet);
        } catch (error) {
            console.error('Context building failed:', error);
            systemPrompt = 'Answer questions about the notebook. Preserve Unicode exactly.';
        }

        // Generate response
        this.updateStatus('GENERATING');
        this.elements.sendBtn.disabled = true;
        this.elements.stopBtn.disabled = false;

        const assistantContainer = this.addMessage({
            role: 'assistant',
            content: '',
            streaming: true,
        });

        try {
            await this.engine.generateResponse(userMessage, systemPrompt);
        } catch (error) {
            if (error.message !== 'Generation aborted') {
                this.onGenerationError(error.message);
            }
        } finally {
            this.updateStatus('READY');
            this.elements.sendBtn.disabled = false;
            this.elements.stopBtn.disabled = true;
            assistantContainer.dataset.streaming = 'false';
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
            const sanitized = this.sanitizeToken(token);
            bubble.textContent += sanitized;

            // Auto-scroll to bottom
            if (!this.isUserScrolledUp()) {
                this.elements.messagesContainer.scrollTop = this.elements.messagesContainer.scrollHeight;
            }
        }
    }

    /**
     * Sanitize token for display (prevent XSS)
     */
    sanitizeToken(token) {
        // Remove control characters but preserve Unicode
        return token.replace(/[\x00-\x08\x0B-\x0C\x0E-\x1F\x7F]/g, '');
    }

    /**
     * Add message to UI
     */
    addMessage({ role, content, streaming = false }) {
        const messageDiv = document.createElement('div');
        messageDiv.className = `jit-message ${role}`;
        messageDiv.dataset.role = role;
        messageDiv.dataset.streaming = streaming ? 'true' : 'false';

        const bubble = document.createElement('div');
        bubble.className = 'jit-message-bubble';

        // Apply role-specific styling
        if (role === 'system') {
            bubble.style.fontSize = '11px';
            bubble.style.color = 'var(--color-text-secondary)';
            bubble.style.fontStyle = 'italic';
        }

        bubble.textContent = content;
        messageDiv.appendChild(bubble);
        this.elements.messagesContainer.appendChild(messageDiv);

        // Auto-scroll to bottom
        this.elements.messagesContainer.scrollTop = this.elements.messagesContainer.scrollHeight;

        return messageDiv;
    }

    /**
     * Check if user has scrolled up
     */
    isUserScrolledUp() {
        const container = this.elements.messagesContainer;
        return container.scrollTop < container.scrollHeight - container.clientHeight - 100;
    }

    /**
     * Clear all messages
     */
    clearMessages() {
        this.elements.messagesContainer.innerHTML = '';
        if (this.engine) {
            this.engine.clearHistory();
        }
        this.receiptChain.clear();
    }

    /**
     * Stop generation
     */
    stopGeneration() {
        if (this.engine && this.engine.isGenerating) {
            this.engine.stopGeneration();
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

        const badgeClass = statusMap[status] || 'offline';

        // Remove all status classes
        for (const cls of this.elements.statusBadge.classList) {
            if (cls.startsWith('jit-status-badge-')) {
                this.elements.statusBadge.classList.remove(cls);
            }
        }

        this.elements.statusBadge.classList.add(`jit-status-badge`);
        this.elements.statusBadge.classList.add(badgeClass);
        this.elements.statusBadge.textContent = status;
    }

    /**
     * Generation complete handler
     */
    async onGenerationComplete(response) {
        try {
            // Create receipt for response
            const receipt = await this.receiptChain.createReceipt({
                agentId: 'jit-assistant',
                capabilityId: 'generate-response',
                instructionHash: 'utf-8-stream',
                action: 'nlg',
                inputHash: '...',
                outputHash: '...',
                keyVersion: 1,
                signature: '...',
                status: 'sealed',
            });

            console.log('Receipt created:', receipt.receiptId);
        } catch (error) {
            console.error('Receipt creation failed:', error);
        }
    }

    /**
     * Generation error handler
     */
    onGenerationError(error) {
        this.addMessage({
            role: 'system',
            content: `✗ Error: ${error}`,
        });
    }

    /**
     * Inject default styling if not already present
     */
    injectDefaultStyling() {
        // Check if styles already present
        if (document.querySelector('style[data-jit-styles]')) {
            return;
        }

        const style = document.createElement('style');
        style.setAttribute('data-jit-styles', 'true');
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

            .jit-status-badge.offline {
                background: rgba(107, 114, 128, 0.3);
                color: #a0a0a0;
            }

            .jit-status-badge.loading {
                background: rgba(59, 130, 246, 0.3);
                color: #60a5fa;
                animation: jit-pulse 1.5s ease-in-out infinite;
            }

            .jit-status-badge.ready {
                background: rgba(16, 185, 129, 0.3);
                color: #10b981;
            }

            .jit-status-badge.generating {
                background: rgba(0, 217, 255, 0.3);
                color: #00d9ff;
                animation: jit-pulse 1s ease-in-out infinite;
            }

            .jit-status-badge.error {
                background: rgba(239, 68, 68, 0.3);
                color: #ef4444;
            }

            @keyframes jit-pulse {
                0%, 100% { opacity: 0.7; }
                50% { opacity: 1; }
            }
        `;
        document.head.appendChild(style);
    }

    /**
     * Get statistics
     */
    getStats() {
        return {
            isOpen: this.state.isOpen,
            hasModel: this.state.hasModel,
            engineStatus: this.engine?.status || 'OFFLINE',
            messageCount: this.elements.messagesContainer.querySelectorAll('.jit-message').length,
            receiptCount: this.receiptChain?.receipts.length || 0,
        };
    }
}

// Initialize JIT Box on DOM ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.jitBox = new JITBox();
    });
} else {
    window.jitBox = new JITBox();
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = JITBox;
}
