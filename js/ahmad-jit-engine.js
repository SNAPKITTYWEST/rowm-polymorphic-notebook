/**
 * Ahmad JIT Engine — Ollama Backend
 * Uses local Ollama API instead of WebLLM
 * Requires: ollama serve running on http://localhost:11434
 */

class AhmadJITEngine {
    constructor(ollamaUrl = 'http://localhost:11434') {
        this.ollamaUrl = ollamaUrl;
        this.state = 'OFFLINE';
        this.conversationHistory = [];
        this.notebookCells = [];
        this.abortController = null;
        this.currentModel = null;
    }

    async checkOllamaConnection() {
        try {
            const response = await fetch(`${this.ollamaUrl}/api/tags`);
            if (!response.ok) {
                throw new Error(`Ollama returned ${response.status}`);
            }
            const data = await response.json();
            return data;
        } catch (error) {
            console.error('Ollama connection failed:', error);
            return null;
        }
    }

    async initialize(modelName = 'tinyllama') {
        try {
            this.state = 'CHECKING_CONNECTION';
            console.log('Checking Ollama connection at:', this.ollamaUrl);

            // Check if Ollama is running
            const tags = await this.checkOllamaConnection();
            if (!tags) {
                throw new Error(`Ollama not running at ${this.ollamaUrl}. Run: ollama serve`);
            }

            console.log('Ollama connected! Available models:', tags.models?.map(m => m.name));

            // Check if model is available
            const modelAvailable = tags.models && tags.models.some(m => m.name === modelName);
            if (!modelAvailable) {
                throw new Error(`Model '${modelName}' not available. Available: ${tags.models?.map(m => m.name).join(', ')}`);
            }

            this.currentModel = modelName;
            this.state = 'INDEXING';
            this.buildNotebookIndex();

            this.state = 'READY';
            console.log(`Ahmad Bot ready with model: ${modelName}`);
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
            const prompt = `${systemPrompt}\n\nAssistant:`;

            console.log(`Generating with model: ${this.currentModel}`);

            // Call Ollama API with streaming
            const response = await fetch(`${this.ollamaUrl}/api/generate`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    model: this.currentModel,
                    prompt: prompt,
                    stream: true,
                    temperature: 0.2,
                    top_p: 0.9,
                    num_predict: 600,
                }),
                signal: this.abortController.signal,
            });

            if (!response.ok) {
                throw new Error(`Ollama API returned ${response.status}`);
            }

            // Parse streaming response
            const reader = response.body.getReader();
            const decoder = new TextDecoder();

            while (true) {
                const { done, value } = await reader.read();
                if (done || this.abortController.signal.aborted) {
                    break;
                }

                const chunk = decoder.decode(value);
                const lines = chunk.split('\n').filter(l => l.trim());

                for (const line of lines) {
                    try {
                        const json = JSON.parse(line);
                        if (json.response) {
                            fullResponse += json.response;
                            if (onToken && typeof onToken === 'function') {
                                try {
                                    onToken(json.response);
                                } catch (callbackError) {
                                    console.error('onToken callback error:', callbackError);
                                }
                            }
                        }
                    } catch (e) {
                        // Skip invalid JSON lines
                    }
                }
            }

            // Store in conversation history
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
        // Ollama stays running, just reset state
        this.currentModel = null;
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
