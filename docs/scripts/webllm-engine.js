/**
 * WebLLM Engine
 * Browser-native model inference with WebGPU acceleration
 */

class WebLLMEngine {
    constructor(options = {}) {
        this.status = 'OFFLINE';
        this.model = options.model || 'TinyLlama-1.1B-Chat-v0.4-q4f16_1-1k';
        this.temperature = options.temperature || 0.2;
        this.topP = options.topP || 0.9;
        this.maxTokens = options.maxTokens || 256;

        this.engine = null;
        this.isLoading = false;
        this.isGenerating = false;
        this.abortController = null;

        this.supportedModels = [
            { id: 'TinyLlama-1.1B-Chat-v0.4-q4f16_1-1k', name: 'TinyLlama 1.1B' },
            { id: 'Mistral-7B-Instruct-v0.2-q4f16_1-MLC', name: 'Mistral 7B' },
            { id: 'Llama-2-7b-chat-hf-q4f32_1-MLC', name: 'Llama 2 7B' },
        ];

        this.conversationHistory = [];
        this.maxHistoryLength = 10;
    }

    /**
     * Initialize WebLLM engine
     */
    async initialize() {
        if (this.status === 'READY' || this.isLoading) {
            return;
        }

        this.isLoading = true;
        this.status = 'LOADING';
        this.emit('statusChanged', 'LOADING');

        try {
            // Check WebLLM availability
            if (typeof window.webllm === 'undefined') {
                throw new Error('WebLLM not loaded. Include @mlc-ai/web-llm script.');
            }

            // Detect WebGPU support
            const hasWebGPU = !!(navigator.gpu);
            console.log(`WebGPU available: ${hasWebGPU}`);

            // Initialize engine
            const webllm = window.webllm;
            this.engine = new webllm.Engine({
                model: this.model,
                useWebGPU: hasWebGPU,
                maxSequenceLength: 4096,
            });

            await this.engine.forward('');

            this.status = 'READY';
            this.isLoading = false;
            this.emit('statusChanged', 'READY');
            console.log(`WebLLM engine initialized: ${this.model}`);
        } catch (error) {
            this.status = 'ERROR';
            this.isLoading = false;
            this.emit('statusChanged', 'ERROR');
            this.emit('error', error.message);
            console.error('WebLLM initialization failed:', error);
            throw error;
        }
    }

    /**
     * Generate response with streaming
     */
    async generateResponse(userMessage, systemPrompt = '') {
        if (this.status !== 'READY' || !this.engine) {
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
            // Build conversation
            const messages = this.buildConversation(userMessage, systemPrompt);

            // Stream generation
            const generator = await this.engine.generate(messages, {
                temperature: this.temperature,
                top_p: this.topP,
                max_new_tokens: this.maxTokens,
            });

            for await (const token of generator) {
                if (this.abortController.signal.aborted) {
                    break;
                }

                fullResponse += token;
                this.emit('token', token);
            }

            // Add to conversation history
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

            return fullResponse;
        } catch (error) {
            if (error.name !== 'AbortError') {
                this.status = 'ERROR';
                this.emit('statusChanged', 'ERROR');
                this.emit('error', error.message);
                console.error('Generation failed:', error);
                throw error;
            }
        } finally {
            this.isGenerating = false;
        }
    }

    /**
     * Build conversation history for API
     */
    buildConversation(userMessage, systemPrompt) {
        const messages = [];

        if (systemPrompt) {
            messages.push({
                role: 'system',
                content: systemPrompt,
            });
        }

        // Add conversation history (bounded)
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
     * Add message to history
     */
    addToHistory(message) {
        this.conversationHistory.push(message);

        // Trim history to max length
        if (this.conversationHistory.length > this.maxHistoryLength * 2) {
            this.conversationHistory = this.conversationHistory.slice(-this.maxHistoryLength);
        }
    }

    /**
     * Stop current generation
     */
    stopGeneration() {
        if (this.abortController) {
            this.abortController.abort();
            this.isGenerating = false;
            this.status = 'READY';
            this.emit('statusChanged', 'READY');
            this.emit('generationStopped');
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
     * Change temperature
     */
    setTemperature(temp) {
        this.temperature = Math.max(0, Math.min(2, temp));
    }

    /**
     * Change top-p
     */
    setTopP(p) {
        this.topP = Math.max(0, Math.min(1, p));
    }

    /**
     * Change max tokens
     */
    setMaxTokens(tokens) {
        this.maxTokens = Math.max(1, Math.min(2048, tokens));
    }

    /**
     * Get current settings
     */
    getSettings() {
        return {
            model: this.model,
            temperature: this.temperature,
            topP: this.topP,
            maxTokens: this.maxTokens,
            status: this.status,
        };
    }

    /**
     * Get supported models
     */
    getSupportedModels() {
        return this.supportedModels;
    }

    /**
     * Switch model
     */
    async switchModel(modelId) {
        if (this.isGenerating) {
            throw new Error('Cannot switch model while generating');
        }

        const supported = this.supportedModels.some(m => m.id === modelId);
        if (!supported) {
            throw new Error(`Model not supported: ${modelId}`);
        }

        this.model = modelId;
        this.status = 'OFFLINE';

        // Reset engine to force re-initialization
        this.engine = null;
        this.emit('statusChanged', 'OFFLINE');
    }

    /**
     * Event emitter
     */
    listeners = {};

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
                    console.error(`Error in listener for ${event}:`, error);
                }
            }
        }
    }

    /**
     * Check WebGPU support
     */
    static hasWebGPU() {
        return !!navigator.gpu;
    }

    /**
     * Check WebLLM availability
     */
    static hasWebLLM() {
        return typeof window.webllm !== 'undefined';
    }

    /**
     * Estimate download size
     */
    estimateModelSize(modelId) {
        const sizes = {
            'TinyLlama-1.1B-Chat-v0.4-q4f16_1-1k': '600MB',
            'Mistral-7B-Instruct-v0.2-q4f16_1-MLC': '4GB',
            'Llama-2-7b-chat-hf-q4f32_1-MLC': '8GB',
        };
        return sizes[modelId] || 'Unknown';
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = WebLLMEngine;
}
