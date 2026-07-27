/**
 * Notebook Context Builder
 * Extracts notebook state for WebLLM and Tau Prolog
 */

class NotebookContextBuilder {
    constructor() {
        this.maxContextTokens = 4000; // Conservative limit
        this.maxCellSummary = 500;
    }

    /**
     * Extract notebook cells from DOM
     */
    extractCells() {
        const cellElements = document.querySelectorAll('.notebook-cell');
        const cells = [];

        cellElements.forEach((elem, index) => {
            const source = elem.querySelector('.cell-editor textarea')?.value || '';
            const output = elem.querySelector('.cell-output')?.textContent || '';
            const cellType = elem.dataset.cellType || 'code';

            cells.push({
                index,
                type: cellType,
                source: source.substring(0, this.maxCellSummary),
                output: output.substring(0, this.maxCellSummary),
                hash: this.hashCell(source, output),
            });
        });

        return cells;
    }

    /**
     * Simple hash for cell identification
     */
    hashCell(source, output) {
        const combined = source + output;
        let hash = 0;
        for (let i = 0; i < combined.length; i++) {
            const char = combined.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32bit integer
        }
        return Math.abs(hash).toString(16);
    }

    /**
     * Build notebook summary
     */
    buildNotebookSummary() {
        const notebookTitle = document.querySelector('#notebook-header h1')?.textContent || 'Untitled';
        const cells = this.extractCells();

        const summary = {
            title: notebookTitle,
            cellCount: cells.length,
            cells: cells,
            recentCells: cells.slice(-3), // Last 3 cells
        };

        return summary;
    }

    /**
     * Build context packet for LLM
     */
    buildContextPacket(userQuestion, selectedCellIndex = null) {
        const summary = this.buildNotebookSummary();
        let tokenCount = 0;

        const packet = {
            question: userQuestion,
            notebookTitle: summary.title,
            cells: [],
            recentOutputs: [],
            trustPolicies: this.extractTrustPolicies(),
            receiptSummary: this.extractReceiptMetadata(),
        };

        // Add nearby cells
        for (const cell of summary.cells) {
            if (selectedCellIndex !== null && Math.abs(cell.index - selectedCellIndex) > 5) {
                continue; // Skip distant cells
            }

            const cellText = `Cell ${cell.index} (${cell.type}):\n${cell.source}\nOutput: ${cell.output}\n`;
            tokenCount += cellText.length / 4; // Rough token estimate

            if (tokenCount < this.maxContextTokens) {
                packet.cells.push({
                    index: cell.index,
                    type: cell.type,
                    source: cell.source,
                    output: cell.output,
                });
            }
        }

        // Add recent outputs for context
        for (const cell of summary.recentCells) {
            if (cell.output) {
                packet.recentOutputs.push({
                    cellIndex: cell.index,
                    output: cell.output.substring(0, 200),
                });
            }
        }

        return packet;
    }

    /**
     * Extract trust policies from sidebar
     */
    extractTrustPolicies() {
        const trustElements = document.querySelectorAll('#trust-list div');
        const policies = [];

        trustElements.forEach(elem => {
            const text = elem.textContent.trim();
            if (text) {
                policies.push(text);
            }
        });

        return policies;
    }

    /**
     * Extract WORM receipt metadata
     */
    extractReceiptMetadata() {
        const receiptElements = document.querySelectorAll('.cell-receipt');
        const metadata = {
            totalReceipts: receiptElements.length,
            recentReceipts: [],
        };

        // Get last 3 receipt hashes
        Array.from(receiptElements)
            .slice(-3)
            .forEach(elem => {
                const hash = elem.dataset.hash || 'unknown';
                const status = elem.dataset.status || 'sealed';
                metadata.recentReceipts.push({ hash: hash.substring(0, 16), status });
            });

        return metadata;
    }

    /**
     * Format context for system prompt
     */
    formatSystemPrompt(packet) {
        let prompt = `You are the assistant for the Isomorphic WORM Notebook.\n\n`;
        prompt += `Notebook: "${packet.notebookTitle}"\n`;
        prompt += `Total cells: ${packet.cells.length}\n`;
        prompt += `Recent receipts: ${packet.receiptSummary.totalReceipts}\n\n`;

        if (packet.cells.length > 0) {
            prompt += `Visible cells:\n`;
            for (const cell of packet.cells) {
                prompt += `\n[Cell ${cell.index}] ${cell.type}\n`;
                prompt += `Source: ${cell.source.substring(0, 100)}...\n`;
                if (cell.output) {
                    prompt += `Output: ${cell.output.substring(0, 100)}...\n`;
                }
            }
        }

        if (packet.trustPolicies.length > 0) {
            prompt += `\nTrust policies:\n`;
            for (const policy of packet.trustPolicies) {
                prompt += `- ${policy}\n`;
            }
        }

        prompt += `\nUser question: ${packet.question}\n`;
        prompt += `Please answer concisely, cite cell references (Cell N), and preserve Unicode exactly.`;

        return prompt;
    }

    /**
     * Estimate token count
     */
    estimateTokens(text) {
        // Rough estimate: ~1 token per 4 characters
        return Math.ceil(text.length / 4);
    }

    /**
     * Validate context packet size
     */
    validateContextSize(packet) {
        const systemPrompt = this.formatSystemPrompt(packet);
        const tokenCount = this.estimateTokens(systemPrompt);

        return {
            valid: tokenCount < this.maxContextTokens,
            tokenCount: tokenCount,
            maxTokens: this.maxContextTokens,
            warning: tokenCount > this.maxContextTokens * 0.8 ? 'Context approaching limit' : null,
        };
    }

    /**
     * Extract cell by index
     */
    getCellByIndex(index) {
        const cells = this.extractCells();
        return cells.find(c => c.index === index);
    }

    /**
     * Get dependency graph (cells that reference each other)
     */
    buildDependencyGraph() {
        const cells = this.extractCells();
        const graph = {};

        cells.forEach(cell => {
            graph[cell.index] = [];

            // Simple heuristic: look for cell references in source
            const cellReferences = cell.source.match(/Cell\s*(\d+)/gi) || [];
            for (const ref of cellReferences) {
                const refIndex = parseInt(ref.match(/\d+/)[0]);
                if (refIndex !== cell.index && cells.some(c => c.index === refIndex)) {
                    graph[cell.index].push(refIndex);
                }
            }
        });

        return graph;
    }

    /**
     * Export context as JSON (for debugging)
     */
    exportContext(userQuestion, selectedCellIndex = null) {
        const packet = this.buildContextPacket(userQuestion, selectedCellIndex);
        const systemPrompt = this.formatSystemPrompt(packet);
        const validation = this.validateContextSize(packet);

        return {
            timestamp: new Date().toISOString(),
            notebook: packet.notebookTitle,
            question: userQuestion,
            systemPrompt: systemPrompt,
            contextPacket: packet,
            validation: validation,
            dependencyGraph: this.buildDependencyGraph(),
        };
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = NotebookContextBuilder;
}
