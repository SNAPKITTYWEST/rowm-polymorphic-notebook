/**
 * Ahmad JIT Engine
 * Real WebLLM local inference for ROWM Notebook
 */

class AhmadJITEngine {
    constructor() {
        this.engine = null;
        this.state = 'OFFLINE';
        this.conversationHistory = [];
        this.notebookCells = [];
        this.abortController = null;
    }

    async initialize(modelId) {
        try {
            this.state = 'CHECKING_WEBGPU';
            if (!navigator.gpu) {
                console.warn('WebGPU not available, will use CPU (10-20x slower)');
            } else {
                console.log('WebGPU available for acceleration');
            }

            this.state = 'LOADING';
            const webllm = window.webllm;
            if (!webllm) {
                // Provide detailed error info
                const errorDetails = [];
                errorDetails.push('WebLLM library not loaded');
                if (window.webllmError) errorDetails.push(`Reason: ${window.webllmError}`);
                if (window.webllmCDNLoading) errorDetails.push('CDN is still loading, please wait');
                const fullError = errorDetails.join(' | ');
                throw new Error(fullError);
            }

            // Log WebLLM version info
            console.log('WebLLM detected. Version:', window.webllm.version || 'unknown');

            // WebLLM 0.2.32 uses new MLCEngine()
            if (typeof webllm.MLCEngine !== 'function') {
                throw new Error('WebLLM MLCEngine not available. Loaded script may be corrupted.');
            }

            // Initialize MLCEngine (no constructor params needed for 0.2.32)
            this.engine = new webllm.MLCEngine();
            console.log('MLCEngine created successfully');

            // Load the model (this is async and downloads weights)
            try {
                console.log(`Downloading model: ${modelId}`);
                await this.engine.reload(modelId);
                console.log(`Model loaded: ${modelId}`);
            } catch (reloadError) {
                console.error('Model reload failed:', reloadError);
                throw new Error(`Failed to load model ${modelId}: ${reloadError.message}`);
            }

            this.state = 'INDEXING';
            this.buildNotebookIndex();

            this.state = 'READY';
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
            if (elem.contains(document.getElementById('ahmad-bot-panel'))) {
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

            // WebLLM 0.2.32 generate with streaming
            let response;
            try {
                response = await this.engine.generate(messages, {
                    temperature: 0.2,
                    top_p: 0.9,
                    max_tokens: 600,
                    stream: true,
                });
            } catch (generateError) {
                this.state = 'ERROR';
                throw new Error(`Generation failed: ${generateError.message}`);
            }

            // Handle streaming response
            if (!response || typeof response[Symbol.asyncIterator] !== 'function') {
                throw new Error('Invalid response from engine: not an async iterable');
            }

            for await (const chunk of response) {
                if (this.abortController.signal.aborted) {
                    break;
                }

                // WebLLM 0.2.32 returns chunks with either text or delta.text
                const token = chunk?.text ?? chunk?.delta?.text ?? chunk?.delta?.content ?? '';
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

            // Store in conversation history (safe due to textContent usage in UI)
            this.conversationHistory.push({ role: 'user', content: userMessage });
            this.conversationHistory.push({ role: 'assistant', content: fullResponse });

            this.state = 'READY';
            return fullResponse;
        } catch (error) {
            if (error.name === 'AbortError' || error.message === 'The operation was aborted') {
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
