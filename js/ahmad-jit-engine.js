/**
 * Ahmad JIT Engine
 * Browser-based LLM inference using WebLLM
 * Model: TinyLlama-1.1B-Chat-v1.0-q4f32_1-MLC (real WebLLM model)
 */

class AhmadJITEngine {
    constructor() {
        this.engine = null;
        this.state = 'OFFLINE';
        this.conversationHistory = [];
        this.notebookCells = [];
        this.abortController = null;
        this.modelId = 'TinyLlama-1.1B-Chat-v1.0-q4f32_1-MLC';  // Real WebLLM model
    }

    async initialize() {
        try {
            this.state = 'CHECKING_WEBGPU';
            if (!navigator.gpu) {
                console.warn('WebGPU not available, will use CPU (slower)');
            } else {
                console.log('WebGPU available for acceleration');
            }

            this.state = 'LOADING';
            const webllm = window.webllm;
            if (!webllm) {
                throw new Error('WebLLM library not loaded. Check CDN script.');
            }

            console.log('✓ window.webllm loaded successfully (no storage access)');
            console.log('✓ WebLLM version:', typeof webllm.version !== 'undefined' ? webllm.version : 'unknown');

            // WebLLM 0.2.32 uses new MLCEngine()
            if (typeof webllm.MLCEngine !== 'function') {
                throw new Error('WebLLM MLCEngine not available');
            }

            console.log('✓ MLCEngine constructor available');
            console.log(`Initializing WebLLM engine with model: ${this.modelId}`);

            // Initialize MLCEngine with in-memory caching (no IndexedDB)
            // This prevents storage tracking issues while maintaining performance
            this.engine = new webllm.MLCEngine({
                model: this.modelId,
                useIndexedDBCache: false,  // CRITICAL: disable IndexedDB storage access
                preferredDevice: 'webgpu'  // Try WebGPU, fallback to WASM
            });
            console.log('MLCEngine created with in-memory caching (no IndexedDB)');

            // Load the model (this downloads ~500MB weights)
            try {
                this.state = 'DOWNLOADING_MODEL';
                console.log(`Downloading model weights for ${this.modelId}...`);
                await this.engine.reload(this.modelId, {
                    initProgressCallback: (msg) => {
                        console.log('Model init progress:', msg);
                    },
                });
                console.log('Model loaded successfully (in-memory, will be lost on page refresh)');
            } catch (reloadError) {
                console.error('Model reload failed:', reloadError);
                throw new Error(`Failed to load model ${this.modelId}: ${reloadError.message}`);
            }

            this.state = 'INDEXING';
            this.buildNotebookIndex();

            this.state = 'READY';
            console.log('Ahmad Bot ready!');
            return true;
        } catch (error) {
            this.state = 'ERROR';
            console.error('Ahmad JIT Engine initialization failed:', error);
            throw error;
        }
    }

    buildNotebookIndex() {
        this.notebookCells = [];
        const cellElements = document.querySelectorAll(
            '[data-cell-id], .notebook-cell, .cell, [class*="cell"]'
        );

        let cellIndex = 0;
        cellElements.forEach((elem) => {
            if (elem.contains(document.getElementById('ahmad-jit-box'))) {
                return; // Skip Ahmad Bot's own UI
            }

            const cellId = elem.dataset.cellId || `ROWM-${String(cellIndex + 1).padStart(3, '0')}`;
            elem.dataset.cellId = cellId;

            const text = elem.textContent.substring(0, 500).trim();
            if (text.length > 0) {
                this.notebookCells.push({
                    id: cellId,
                    text: text,
                    element: elem,
                });
                cellIndex++;
            }
        });

        console.log(`Indexed ${this.notebookCells.length} notebook cells`);
    }

    findRelevantCells(question, maxCells = 3) {
        const questionTerms = question.toLowerCase().split(/\s+/);
        const scored = this.notebookCells.map((cell) => {
            const cellLower = cell.text.toLowerCase();
            const matches = questionTerms.filter((term) => cellLower.includes(term)).length;
            return { cell, score: matches };
        });

        return scored
            .filter((s) => s.score > 0)
            .sort((a, b) => b.score - a.score)
            .slice(0, maxCells)
            .map((s) => s.cell);
    }

    buildSystemPrompt(question) {
        const relevant = this.findRelevantCells(question);
        let context = 'NOTEBOOK CONTEXT:\n\n';

        relevant.forEach((cell) => {
            context += `[${cell.id}]\n${cell.text}\n\n`;
        });

        return `You are Ahmad Bot, the embedded local assistant for the ROWM Polymorphic Notebook.

Answer from the supplied notebook cells.

Cite notebook cells using their real identifiers.

Do not invent missing information.

Preserve Unicode exactly.

Do not claim a hash is a signature.

Do not claim visual animation is formal verification.

${context}

USER QUESTION:
${question}`;
    }

    async generate(userMessage, onToken) {
        if (this.state !== 'READY') {
            throw new Error(`Engine not ready: ${this.state}`);
        }

        if (!userMessage || userMessage.trim().length === 0) {
            throw new Error('User message cannot be empty');
        }

        this.state = 'GENERATING';
        this.abortController = new AbortController();
        let fullResponse = '';

        try {
            const systemPrompt = this.buildSystemPrompt(userMessage);
            const messages = [
                { role: 'system', content: systemPrompt },
                ...this.conversationHistory,
                { role: 'user', content: userMessage },
            ];

            console.log('Generating response...');

            // WebLLM generate() returns async iterable of response chunks
            const response = await this.engine.generate(messages, {
                temperature: 0.2,
                top_p: 0.9,
                max_tokens: 600,
                stream: true,
            });

            // Stream tokens from response
            for await (const chunk of response) {
                if (this.abortController.signal.aborted) {
                    break;
                }

                // WebLLM chunk has format: { text: "token" } or { delta: { text: "token" } }
                const token = chunk.text ?? chunk.delta?.text ?? '';
                if (token && typeof token === 'string') {
                    fullResponse += token;
                    if (onToken && typeof onToken === 'function') {
                        try {
                            onToken(token);
                        } catch (callbackError) {
                            console.error('onToken callback error:', callbackError);
                        }
                    }
                }
            }

            // Store in conversation history
            this.conversationHistory.push({ role: 'user', content: userMessage });
            this.conversationHistory.push({ role: 'assistant', content: fullResponse });

            this.state = 'READY';
            return fullResponse;
        } catch (error) {
            if (error.name === 'AbortError') {
                this.state = 'STOPPED';
                return fullResponse;
            }
            this.state = 'ERROR';
            console.error('Generation error:', error);
            throw error;
        }
    }

    stop() {
        if (this.abortController) {
            this.abortController.abort();
        }
    }

    resetConversation() {
        this.conversationHistory = [];
    }

    unload() {
        if (this.engine) {
            this.engine.terminate?.();
        }
        this.engine = null;
        this.state = 'OFFLINE';
    }

    getState() {
        return this.state;
    }
}

// Global instance
window.ahmadEngine = null;

// Export
if (typeof module !== 'undefined' && module.exports) {
    module.exports = AhmadJITEngine;
}
