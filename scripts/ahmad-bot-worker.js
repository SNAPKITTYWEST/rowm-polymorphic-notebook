/**
 * Ahmad Bot Web Worker
 * Offloads model inference to prevent UI blocking
 * Communicates with main thread via message passing
 */

let engine = null;
let isInitializing = false;

/**
 * Handle messages from main thread
 */
self.onmessage = async (event) => {
    const { type, data } = event.data;

    try {
        switch (type) {
            case 'INIT':
                await handleInit(data);
                break;
            case 'GENERATE':
                await handleGenerate(data);
                break;
            case 'STOP':
                handleStop();
                break;
            case 'STATUS':
                handleStatus();
                break;
            default:
                console.warn(`[Ahmad Bot Worker] Unknown message type: ${type}`);
        }
    } catch (error) {
        self.postMessage({
            type: 'ERROR',
            error: error.message,
            stack: error.stack,
        });
    }
};

/**
 * Initialize WebLLM engine
 */
async function handleInit(data) {
    if (isInitializing || engine) {
        self.postMessage({
            type: 'INIT_ACK',
            status: 'ALREADY_INITIALIZED',
        });
        return;
    }

    isInitializing = true;
    self.postMessage({
        type: 'INIT_PROGRESS',
        message: 'Initializing WebLLM engine...',
    });

    try {
        // Import WebLLM if available
        const { modelId = 'Llama-2-7b-chat-hf-q4f32_1-MLC' } = data;

        // Check if WebLLM is available in worker
        if (typeof self.webllm === 'undefined') {
            throw new Error('WebLLM not available in worker. Ensure script is imported.');
        }

        // Initialize engine
        engine = new self.webllm.MLCEngine({
            model: modelId,
            temperature: 0.3,
            top_p: 0.85,
            max_gen_len: 512,
        });

        // Wait for ready
        await engine.ready;

        isInitializing = false;

        self.postMessage({
            type: 'INIT_ACK',
            status: 'READY',
            model: modelId,
        });

    } catch (error) {
        isInitializing = false;
        throw error;
    }
}

/**
 * Generate response with streaming
 */
async function handleGenerate(data) {
    if (!engine) {
        throw new Error('Engine not initialized');
    }

    const { messages, requestId } = data;

    try {
        self.postMessage({
            type: 'GENERATION_START',
            requestId: requestId,
        });

        let fullResponse = '';
        let tokenCount = 0;

        // Create stream
        const stream = await engine.chat.completions.create({
            messages: messages,
            stream: true,
            temperature: 0.3,
            top_p: 0.85,
        });

        // Consume stream and send tokens
        for await (const chunk of stream) {
            const token = chunk.choices?.[0]?.delta?.content || '';

            if (token) {
                fullResponse += token;
                tokenCount++;

                // Send token to main thread
                self.postMessage({
                    type: 'TOKEN',
                    requestId: requestId,
                    token: token,
                    tokenCount: tokenCount,
                });

                // Yield to prevent blocking
                if (tokenCount % 10 === 0) {
                    await new Promise(r => setTimeout(r, 0));
                }
            }
        }

        // Send completion
        self.postMessage({
            type: 'GENERATION_COMPLETE',
            requestId: requestId,
            fullResponse: fullResponse,
            tokenCount: tokenCount,
        });

    } catch (error) {
        self.postMessage({
            type: 'GENERATION_ERROR',
            requestId: data.requestId,
            error: error.message,
        });
    }
}

/**
 * Stop current generation
 */
function handleStop() {
    if (engine) {
        engine.interruptGenerate?.();
    }

    self.postMessage({
        type: 'GENERATION_STOPPED',
    });
}

/**
 * Get engine status
 */
function handleStatus() {
    self.postMessage({
        type: 'STATUS_ACK',
        isReady: engine !== null,
        isInitializing: isInitializing,
    });
}

// Signal to main thread that worker is ready
self.postMessage({
    type: 'WORKER_READY',
});
