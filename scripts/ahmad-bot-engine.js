/**
 * Ahmad Bot Engine
 * Real WebLLM model integration with notebook context extraction
 * Produces genuine model inference, not mocked responses
 */

class NotebookPageReader {
    /**
     * Extract all notebook cells from DOM, ignoring nav/buttons
     */
    static extractCells() {
        const cells = [];
        const cellElements = document.querySelectorAll('.notebook-cell');

        cellElements.forEach((elem, index) => {
            const sourceElem = elem.querySelector('.cell-editor textarea');
            const outputElem = elem.querySelector('.cell-output');
            const indexElem = elem.querySelector('.cell-index');

            if (!sourceElem) return;

            const cellId = `cell_${index}`;
            const cellType = elem.dataset.cellType || 'code';
            const source = sourceElem.value || '';
            const output = outputElem?.textContent || '';
            const indexText = indexElem?.textContent || `Cell [${index}]`;

            cells.push({
                id: cellId,
                index: index,
                type: cellType,
                source: source.trim(),
                output: output.trim(),
                indexText: indexText,
                hash: this.hashContent(source + output),
            });
        });

        return cells;
    }

    /**
     * Simple hash for cell identification
     */
    static hashContent(content) {
        let hash = 0;
        for (let i = 0; i < content.length; i++) {
            const char = content.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash;
        }
        return Math.abs(hash).toString(16);
    }

    /**
     * Get notebook metadata
     */
    static getNotebookMetadata() {
        return {
            title: document.querySelector('#notebook-header h1')?.textContent || 'ROWM Notebook',
            subtitle: document.querySelector('.subtitle')?.textContent || '',
            cellCount: document.querySelectorAll('.notebook-cell').length,
            timestamp: new Date().toISOString(),
        };
    }
}

class NotebookContextIndex {
    /**
     * Build searchable index of notebook cells
     */
    constructor() {
        this.cells = [];
        this.metadata = {};
        this.index = {};
        this.rebuild();
    }

    /**
     * Rebuild index from current notebook state
     */
    rebuild() {
        this.cells = NotebookPageReader.extractCells();
        this.metadata = NotebookPageReader.getNotebookMetadata();
        this.rebuildSearchIndex();
    }

    /**
     * Create search index for relevance retrieval
     */
    rebuildSearchIndex() {
        this.index = {};

        this.cells.forEach(cell => {
            const tokens = this.tokenize(cell.source + ' ' + cell.output);
            tokens.forEach(token => {
                if (!this.index[token]) {
                    this.index[token] = [];
                }
                if (!this.index[token].includes(cell.index)) {
                    this.index[token].push(cell.index);
                }
            });
        });
    }

    /**
     * Simple tokenization
     */
    tokenize(text) {
        return text
            .toLowerCase()
            .match(/\b[\w]+\b/g)
            .filter((t, i, a) => a.indexOf(t) === i && t.length > 2)
            .slice(0, 50);
    }

    /**
     * Find relevant cells based on query
     */
    findRelevant(query, limit = 5) {
        const tokens = this.tokenize(query);
        const cellScores = {};

        tokens.forEach(token => {
            const cellIndices = this.index[token] || [];
            cellIndices.forEach(idx => {
                cellScores[idx] = (cellScores[idx] || 0) + 1;
            });
        });

        // Sort by relevance and return cells
        const sorted = Object.entries(cellScores)
            .sort((a, b) => b[1] - a[1])
            .slice(0, limit)
            .map(([idx]) => this.cells[parseInt(idx)])
            .filter(Boolean);

        // If no matches, return recent cells
        if (sorted.length === 0) {
            return this.cells.slice(-3);
        }

        return sorted;
    }

    /**
     * Get cell by index
     */
    getCellByIndex(index) {
        return this.cells.find(c => c.index === index);
    }

    /**
     * Get all cells
     */
    getAllCells() {
        return this.cells;
    }

    /**
     * Export context as text
     */
    formatContextAsText(relevantCells) {
        let text = `ROWM Notebook Context\n`;
        text += `Title: ${this.metadata.title}\n`;
        text += `Total Cells: ${this.metadata.cellCount}\n`;
        text += `===================\n\n`;

        relevantCells.forEach(cell => {
            text += `${cell.indexText}\n`;
            text += `Type: ${cell.type}\n`;
            if (cell.source) {
                text += `Source:\n${cell.source.substring(0, 500)}\n`;
            }
            if (cell.output) {
                text += `Output:\n${cell.output.substring(0, 500)}\n`;
            }
            text += `---\n\n`;
        });

        return text;
    }
}

class AhmadWebLLMEngine {
    constructor(options = {}) {
        this.status = 'OFFLINE';
        this.modelId = options.modelId || 'Llama-2-7b-chat-hf-q4f32_1-MLC';
        this.temperature = options.temperature || 0.3;
        this.topP = options.topP || 0.85;
        this.maxTokens = options.maxTokens || 512;

        this.engine = null;
        this.worker = null;
        this.isLoading = false;
        this.isGenerating = false;
        this.abortController = null;
        this.listeners = {};
        this.contextIndex = new NotebookContextIndex();

        this.supportedModels = [
            { id: 'Llama-2-7b-chat-hf-q4f32_1-MLC', name: 'Llama 2 7B (q4f32)', size: '3.9GB' },
            { id: 'Mistral-7B-Instruct-v0.2-q4f16_1-MLC', name: 'Mistral 7B (q4f16)', size: '4.1GB' },
            { id: 'TinyLlama-1.1B-Chat-v0.4-q4f16_1-1k', name: 'TinyLlama 1.1B (q4f16)', size: '530MB' },
            { id: 'NeuralHermes-2.5-Mistral-7B-q4f16_1-MLC', name: 'NeuralHermes 7B (q4f16)', size: '4.2GB' },
        ];

        this.conversationHistory = [];
        this.maxHistoryLength = 8;
    }

    /**
     * Initialize WebLLM engine
     */
    async initialize(modelId = null) {
        if (modelId) {
            this.modelId = modelId;
        }

        if (this.status === 'READY' || this.isLoading) {
            return;
        }

        this.isLoading = true;
        this.emit('statusChanged', 'LOADING');

        try {
            // Check WebLLM availability
            if (typeof window.webllm === 'undefined') {
                throw new Error('WebLLM not loaded. Include @mlc-ai/web-llm script.');
            }

            console.log(`[Ahmad Bot] Initializing model: ${this.modelId}`);

            // Initialize MLCEngine
            const webllm = window.webllm;
            this.engine = new webllm.MLCEngine({
                model: this.modelId,
                temperature: this.temperature,
                top_p: this.topP,
                max_gen_len: this.maxTokens,
            });

            // Wait for engine ready
            await this.engine.ready;

            this.status = 'READY';
            this.isLoading = false;
            this.emit('statusChanged', 'READY');
            console.log(`[Ahmad Bot] Engine ready: ${this.modelId}`);

            return true;
        } catch (error) {
            this.status = 'ERROR';
            this.isLoading = false;
            this.emit('statusChanged', 'ERROR');
            this.emit('error', error.message);
            console.error('[Ahmad Bot] Initialization failed:', error);
            throw error;
        }
    }

    /**
     * Check if engine is ready
     */
    isReady() {
        return this.status === 'READY' && this.engine !== null;
    }

    /**
     * Generate response with real streaming
     */
    async generate(userMessage, systemPrompt = '') {
        if (!this.isReady()) {
            throw new Error('Engine not ready');
        }

        if (this.isGenerating) {
            throw new Error('Generation already in progress');
        }

        this.isGenerating = true;
        this.status = 'GENERATING';
        this.emit('statusChanged', 'GENERATING');
        this.emit('generationStart');

        this.abortController = new AbortController();
        let fullResponse = '';

        try {
            // Build actual conversation
            const messages = this.buildConversation(userMessage, systemPrompt);

            console.log('[Ahmad Bot] Generating response...');

            // Stream from real model
            const stream = await this.engine.chat.completions.create({
                messages: messages,
                stream: true,
                temperature: this.temperature,
                top_p: this.topP,
            });

            // Consume stream
            for await (const chunk of stream) {
                if (this.abortController.signal.aborted) {
                    throw new Error('Generation aborted');
                }

                const token = chunk.choices?.[0]?.delta?.content || '';
                if (token) {
                    fullResponse += token;
                    this.emit('token', token);
                }
            }

            // Add to history
            this.addToHistory({
                role: 'user',
                content: userMessage,
            });

            this.addToHistory({
                role: 'assistant',
                content: fullResponse,
            });

            this.status = 'READY';
            this.emit('statusChanged', 'READY');
            this.emit('generationComplete', fullResponse);

            console.log('[Ahmad Bot] Generation complete');
            return fullResponse;

        } catch (error) {
            if (error.name === 'AbortError' || error.message.includes('aborted')) {
                console.log('[Ahmad Bot] Generation aborted by user');
                this.emit('generationStopped');
            } else {
                this.status = 'ERROR';
                this.emit('statusChanged', 'ERROR');
                this.emit('error', error.message);
                console.error('[Ahmad Bot] Generation error:', error);
                throw error;
            }
        } finally {
            this.isGenerating = false;
        }
    }

    /**
     * Build conversation with context from notebook
     */
    buildConversation(userMessage, systemPrompt) {
        const messages = [];

        // System prompt with context
        if (systemPrompt) {
            messages.push({
                role: 'system',
                content: systemPrompt,
            });
        }

        // Add conversation history
        for (const msg of this.conversationHistory.slice(-this.maxHistoryLength)) {
            messages.push(msg);
        }

        // Add current message
        messages.push({
            role: 'user',
            content: userMessage,
        });

        return messages;
    }

    /**
     * Build system prompt with notebook context
     */
    buildSystemPrompt(userMessage) {
        // Refresh context from notebook
        this.contextIndex.rebuild();

        // Find relevant cells
        const relevantCells = this.contextIndex.findRelevant(userMessage, 5);
        const contextText = this.contextIndex.formatContextAsText(relevantCells);

        const prompt = `You are Ahmad Bot, an embedded technical guide for the Isomorphic WORM Notebook running locally in the browser.

You have access to the following notebook context:

${contextText}

Instructions:
- Answer questions based on notebook content
- Cite cell identifiers (e.g., "Cell 0", "Cell 1")
- Never invent cells or content
- Preserve Unicode exactly (λ Ω ϕ ∑ 𐤀 ꙮ)
- Be concise and direct
- If uncertain about content, say so

User question: ${userMessage}`;

        return prompt;
    }

    /**
     * Add message to history
     */
    addToHistory(message) {
        this.conversationHistory.push(message);

        // Trim history
        if (this.conversationHistory.length > this.maxHistoryLength * 2) {
            this.conversationHistory = this.conversationHistory.slice(-this.maxHistoryLength);
        }
    }

    /**
     * Interrupt generation
     */
    interrupt() {
        if (this.isGenerating && this.abortController) {
            this.abortController.abort();
            this.isGenerating = false;
            this.status = 'READY';
            this.emit('statusChanged', 'READY');
        }
    }

    /**
     * Clear conversation history
     */
    clearHistory() {
        this.conversationHistory = [];
        this.emit('historyCleared');
    }

    /**
     * Get supported models
     */
    getSupportedModels() {
        return this.supportedModels;
    }

    /**
     * Get current status
     */
    getStatus() {
        return {
            status: this.status,
            model: this.modelId,
            isReady: this.isReady(),
            isGenerating: this.isGenerating,
            temperature: this.temperature,
            topP: this.topP,
            maxTokens: this.maxTokens,
        };
    }

    /**
     * Event emitter
     */
    on(event, callback) {
        if (!this.listeners[event]) {
            this.listeners[event] = [];
        }
        this.listeners[event].push(callback);
    }

    off(event, callback) {
        if (this.listeners[event]) {
            this.listeners[event] = this.listeners[event].filter(cb => cb !== callback);
        }
    }

    emit(event, data) {
        if (this.listeners[event]) {
            for (const callback of this.listeners[event]) {
                try {
                    callback(data);
                } catch (error) {
                    console.error(`[Ahmad Bot] Error in listener for ${event}:`, error);
                }
            }
        }
    }

    /**
     * Static: Check WebLLM availability
     */
    static hasWebLLM() {
        return typeof window.webllm !== 'undefined';
    }

    /**
     * Static: Check WebGPU support
     */
    static hasWebGPU() {
        return !!navigator.gpu;
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { AhmadWebLLMEngine, NotebookContextIndex, NotebookPageReader };
}
