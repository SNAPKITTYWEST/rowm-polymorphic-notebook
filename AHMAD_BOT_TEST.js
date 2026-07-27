/**
 * Ahmad Bot Integration Test
 * Run this in the browser console to verify full setup
 *
 * Usage:
 * 1. Open index-app.html
 * 2. Open DevTools (F12)
 * 3. Copy-paste this into Console
 * 4. Follow test output
 */

const AhmadBotTest = {
    /**
     * Run all tests
     */
    async runAll() {
        console.clear();
        console.log('%c=== Ahmad Bot Integration Test ===', 'color: #00d9ff; font-weight: bold; font-size: 14px;');
        console.log('');

        let passed = 0;
        let failed = 0;

        // Test 1: Check WebLLM loaded
        console.log('%cTest 1: WebLLM Library', 'color: #ffd700; font-weight: bold;');
        if (typeof window.webllm !== 'undefined') {
            console.log('✓ WebLLM loaded:', window.webllm);
            passed++;
        } else {
            console.error('✗ WebLLM not available');
            failed++;
        }
        console.log('');

        // Test 2: Check Engine class
        console.log('%cTest 2: AhmadWebLLMEngine Class', 'color: #ffd700; font-weight: bold;');
        if (typeof AhmadWebLLMEngine !== 'undefined') {
            console.log('✓ AhmadWebLLMEngine loaded');
            const engine = new AhmadWebLLMEngine();
            console.log('  - Status:', engine.status);
            console.log('  - Model:', engine.modelId);
            console.log('  - Supported models:', engine.getSupportedModels().length);
            passed++;
        } else {
            console.error('✗ AhmadWebLLMEngine not found');
            failed++;
        }
        console.log('');

        // Test 3: Check UI class
        console.log('%cTest 3: AhmadBotUI Class', 'color: #ffd700; font-weight: bold;');
        if (typeof AhmadBotUI !== 'undefined') {
            console.log('✓ AhmadBotUI loaded');
            console.log('  - Global instance:', window.ahmadBotUI);
            passed++;
        } else {
            console.error('✗ AhmadBotUI not found');
            failed++;
        }
        console.log('');

        // Test 4: Check DOM elements
        console.log('%cTest 4: DOM Elements', 'color: #ffd700; font-weight: bold;');
        const elements = {
            launcher: '#jit-launcher',
            button: '#jit-toggle',
            panel: '#jit-panel',
            messages: '#jit-messages',
            input: '#jit-input',
            send: '#jit-send',
        };

        let domPassed = true;
        for (const [name, selector] of Object.entries(elements)) {
            const elem = document.querySelector(selector);
            if (elem) {
                console.log(`✓ ${name}: ${selector}`);
            } else {
                console.error(`✗ ${name}: ${selector} NOT FOUND`);
                domPassed = false;
            }
        }
        if (domPassed) passed++; else failed++;
        console.log('');

        // Test 5: Check NotebookPageReader
        console.log('%cTest 5: Notebook Context Reader', 'color: #ffd700; font-weight: bold;');
        if (typeof NotebookPageReader !== 'undefined') {
            console.log('✓ NotebookPageReader loaded');
            const cells = NotebookPageReader.extractCells();
            console.log(`  - Cells found: ${cells.length}`);
            const meta = NotebookPageReader.getNotebookMetadata();
            console.log(`  - Notebook: "${meta.title}"`);
            if (cells.length > 0) {
                passed++;
            } else {
                console.warn('⚠ No cells found (notebook may be empty)');
            }
        } else {
            console.error('✗ NotebookPageReader not found');
            failed++;
        }
        console.log('');

        // Test 6: Check NotebookContextIndex
        console.log('%cTest 6: Context Index', 'color: #ffd700; font-weight: bold;');
        if (typeof NotebookContextIndex !== 'undefined') {
            console.log('✓ NotebookContextIndex loaded');
            const index = new NotebookContextIndex();
            console.log(`  - Indexed cells: ${index.getAllCells().length}`);
            const relevant = index.findRelevant('Unicode', 3);
            console.log(`  - Relevant cells for "Unicode": ${relevant.length}`);
            passed++;
        } else {
            console.error('✗ NotebookContextIndex not found');
            failed++;
        }
        console.log('');

        // Test 7: CSS loaded
        console.log('%cTest 7: Styles', 'color: #ffd700; font-weight: bold;');
        const styleSheet = Array.from(document.styleSheets).find(s =>
            s.href?.includes('ahmad-bot.css') || s.textContent?.includes('--ahmad-')
        );
        if (styleSheet || document.querySelector('style[data-ahmad-bot-styles]')) {
            console.log('✓ Ahmad Bot CSS loaded');
            const primary = getComputedStyle(document.documentElement).getPropertyValue('--ahmad-primary');
            console.log('  - Theme colors detected');
            passed++;
        } else {
            console.warn('⚠ Ahmad Bot CSS may not be loaded (check stylesheet)');
        }
        console.log('');

        // Test 8: WebGPU support
        console.log('%cTest 8: Hardware Acceleration', 'color: #ffd700; font-weight: bold;');
        const hasWebGPU = AhmadWebLLMEngine?.hasWebGPU?.();
        if (hasWebGPU) {
            console.log('✓ WebGPU available (GPU acceleration enabled)');
        } else {
            console.log('⚠ WebGPU not available (CPU mode will be used)');
        }
        console.log(`  - Navigator.gpu: ${!!navigator.gpu}`);
        passed++;
        console.log('');

        // Summary
        console.log('%c=== Test Summary ===', 'color: #00d9ff; font-weight: bold; font-size: 14px;');
        console.log(`Passed: %c${passed}`, 'color: #10b981; font-weight: bold;');
        console.log(`Failed: %c${failed}`, failed > 0 ? 'color: #ef4444; font-weight: bold;' : 'color: #10b981;');
        console.log('');

        if (failed === 0) {
            console.log('%c✓ All systems ready. Click Ω to start Ahmad Bot!', 'color: #10b981; font-weight: bold; font-size: 12px;');
        } else {
            console.error('%c✗ Some tests failed. Check errors above.', 'color: #ef4444; font-weight: bold;');
        }

        return { passed, failed };
    },

    /**
     * Test UI interaction
     */
    testUI() {
        console.log('%cTesting UI...', 'color: #ffd700; font-weight: bold;');

        if (!window.ahmadBotUI) {
            console.error('UI not initialized');
            return;
        }

        const ui = window.ahmadBotUI;
        console.log('Current state:', ui.getStats());

        // Open panel
        console.log('Opening panel...');
        ui.openPanel();

        console.log('Panel should now be visible. Status should be OFFLINE.');
        console.log('Wait for model to initialize (LOADING → READY)');
        console.log('Then type a message and press Send');
    },

    /**
     * Test engine directly
     */
    async testEngine() {
        console.log('%cTesting Engine...', 'color: #ffd700; font-weight: bold;');

        try {
            const engine = new AhmadWebLLMEngine();

            console.log('Engine created:', engine.status);

            if (!AhmadWebLLMEngine.hasWebLLM()) {
                console.error('WebLLM not available');
                return;
            }

            console.log('WebLLM available, initializing...');
            await engine.initialize('TinyLlama-1.1B-Chat-v0.4-q4f16_1-1k');

            console.log('Engine ready:', engine.isReady());

            if (engine.isReady()) {
                console.log('Engine is ready for generation');
                console.log('Try: engine.generate("Hello", "You are helpful.")');
            }

            return engine;

        } catch (error) {
            console.error('Engine test failed:', error.message);
        }
    },

    /**
     * Print configuration
     */
    printConfig() {
        console.log('%c=== Ahmad Bot Configuration ===', 'color: #00d9ff; font-weight: bold;');

        if (window.ahmadBotUI?.engine) {
            const status = window.ahmadBotUI.engine.getStatus();
            console.table(status);
        } else {
            console.log('Engine not initialized yet');
            console.log('Click Ω and wait for READY status');
        }
    },

    /**
     * Get help
     */
    help() {
        console.log('%c=== Ahmad Bot Test Commands ===', 'color: #00d9ff; font-weight: bold;');
        console.log('');
        console.log('AhmadBotTest.runAll()     — Run full integration test');
        console.log('AhmadBotTest.testUI()     — Test UI interaction');
        console.log('AhmadBotTest.testEngine() — Test engine (async)');
        console.log('AhmadBotTest.printConfig() — Show current config');
        console.log('AhmadBotTest.help()       — Show this help');
        console.log('');
        console.log('Quick start:');
        console.log('1. AhmadBotTest.runAll()');
        console.log('2. AhmadBotTest.testUI()');
        console.log('3. Click Ω button');
        console.log('4. Wait for READY status');
        console.log('5. Type your question');
    }
};

// Auto-run on load
console.log('%c👋 Ahmad Bot Test Suite Loaded', 'color: #00d9ff; font-weight: bold;');
console.log('Run: AhmadBotTest.runAll()');
console.log('Or:  AhmadBotTest.help()');
