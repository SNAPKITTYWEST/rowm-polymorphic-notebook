/**
 * Notebook Engine
 * Core notebook execution, cell management, and state orchestration
 */

class NotebookEngine {
    constructor() {
        this.cells = [];
        this.selectedCell = null;
        this.metadata = {};
        this.initDOM();
    }

    /**
     * Initialize DOM elements
     */
    initDOM() {
        this.elements = {
            cellsContainer: document.getElementById('notebook-cells'),
            statusText: document.getElementById('status-text'),
            executionStats: document.getElementById('execution-stats'),
            metadataList: document.getElementById('metadata-list'),
            dependencySvg: document.getElementById('dependency-svg'),
        };
    }

    /**
     * Add a new cell
     */
    addCell(type = 'code', index = -1) {
        if (index === -1) {
            index = this.cells.length;
        }

        const cell = {
            index: index,
            type: type,
            source: '',
            output: '',
            hash: '',
            executionTime: 0,
            status: 'ready',
        };

        this.cells.push(cell);
        this.renderCell(cell);
        return cell;
    }

    /**
     * Render cell in DOM
     */
    renderCell(cell) {
        const cellDiv = document.createElement('div');
        cellDiv.className = 'notebook-cell';
        cellDiv.dataset.index = cell.index;
        cellDiv.dataset.cellType = cell.type;

        cellDiv.innerHTML = `
            <div class="cell-index">Cell [${cell.index}] • ${cell.type}</div>
            <div class="cell-editor">
                <textarea placeholder="Enter ${cell.type} here..." spellcheck="false"></textarea>
            </div>
            <div class="cell-output"></div>
            <div class="cell-receipt" style="display: none;"></div>
        `;

        // Add event listeners
        const textarea = cellDiv.querySelector('textarea');
        textarea.addEventListener('input', (e) => {
            cell.source = e.target.value;
            this.updateCellHash(cell);
        });

        textarea.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'Enter') {
                this.executeCell(cell.index);
            }
        });

        cellDiv.addEventListener('click', () => {
            this.selectCell(cell.index);
        });

        this.elements.cellsContainer.appendChild(cellDiv);
    }

    /**
     * Select a cell
     */
    selectCell(index) {
        if (this.selectedCell !== null) {
            const prevCell = this.elements.cellsContainer.querySelector(
                `[data-index="${this.selectedCell}"]`
            );
            if (prevCell) {
                prevCell.style.borderColor = '';
            }
        }

        this.selectedCell = index;
        const cellDiv = this.elements.cellsContainer.querySelector(
            `[data-index="${index}"]`
        );
        if (cellDiv) {
            cellDiv.style.borderColor = 'var(--color-cyan)';
        }
    }

    /**
     * Execute a cell
     */
    async executeCell(index) {
        const cell = this.cells[index];
        if (!cell) return;

        cell.status = 'running';
        const startTime = performance.now();

        const cellDiv = this.elements.cellsContainer.querySelector(`[data-index="${index}"]`);
        const outputDiv = cellDiv.querySelector('.cell-output');

        try {
            // Simulate execution (in real implementation, would dispatch to runtime)
            await new Promise(resolve => setTimeout(resolve, 500));

            // Mock output
            cell.output = `Output from cell [${index}]: ${cell.source.substring(0, 50)}...`;
            outputDiv.textContent = cell.output;
            outputDiv.classList.remove('error');
            outputDiv.classList.add('success');

            cell.status = 'success';
        } catch (error) {
            cell.output = `Error: ${error.message}`;
            outputDiv.textContent = cell.output;
            outputDiv.classList.remove('success');
            outputDiv.classList.add('error');

            cell.status = 'error';
        }

        cell.executionTime = performance.now() - startTime;
        this.updateStats();
    }

    /**
     * Execute all cells in order
     */
    async executeAll() {
        this.updateStatus('Executing all cells...');

        for (let i = 0; i < this.cells.length; i++) {
            await this.executeCell(i);
        }

        this.updateStatus('All cells executed');
    }

    /**
     * Update cell hash (for tamper detection)
     */
    updateCellHash(cell) {
        const data = cell.source + cell.output;
        let hash = 0;
        for (let i = 0; i < data.length; i++) {
            const char = data.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
        }
        cell.hash = Math.abs(hash).toString(16);
    }

    /**
     * Update statistics
     */
    updateStats() {
        const totalExecutionTime = this.cells.reduce((sum, c) => sum + c.executionTime, 0);
        const totalCells = this.cells.length;

        this.elements.executionStats.textContent =
            `${totalCells} cells | ${Math.round(totalExecutionTime)}ms`;

        this.updateMetadata();
    }

    /**
     * Update metadata display
     */
    updateMetadata() {
        this.elements.metadataList.innerHTML = '';

        const metadata = [
            { label: 'Cells', value: this.cells.length },
            { label: 'Successful', value: this.cells.filter(c => c.status === 'success').length },
            { label: 'Errors', value: this.cells.filter(c => c.status === 'error').length },
            { label: 'Total time', value: `${Math.round(this.cells.reduce((s, c) => s + c.executionTime, 0))}ms` },
        ];

        for (const item of metadata) {
            const div = document.createElement('div');
            div.innerHTML = `<span>${item.label}:</span> ${item.value}`;
            this.elements.metadataList.appendChild(div);
        }
    }

    /**
     * Update status text
     */
    updateStatus(text) {
        this.elements.statusText.textContent = text;
    }

    /**
     * Render dependency graph
     */
    renderDependencyGraph() {
        const svg = this.elements.dependencySvg;
        if (!svg) return;

        const width = svg.width.baseVal.value;
        const height = svg.height.baseVal.value;

        // Clear previous
        svg.innerHTML = '';

        // Simple graph layout
        const nodeCount = Math.min(this.cells.length, 10);
        const radius = 15;
        const padding = 20;
        const cx = width / 2;
        const cy = height / 2;
        const maxRadius = Math.min(width, height) / 2 - padding;

        // Draw nodes (cells)
        for (let i = 0; i < nodeCount; i++) {
            const angle = (i / nodeCount) * 2 * Math.PI;
            const x = cx + maxRadius * Math.cos(angle);
            const y = cy + maxRadius * Math.sin(angle);

            // Node circle
            const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            circle.setAttribute('cx', x);
            circle.setAttribute('cy', y);
            circle.setAttribute('r', radius);
            svg.appendChild(circle);

            // Label
            const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            text.setAttribute('x', x);
            text.setAttribute('y', y + 3);
            text.textContent = i;
            svg.appendChild(text);
        }

        // Draw connections (simple: each cell to next)
        for (let i = 0; i < nodeCount - 1; i++) {
            const angle1 = (i / nodeCount) * 2 * Math.PI;
            const angle2 = ((i + 1) / nodeCount) * 2 * Math.PI;

            const x1 = cx + maxRadius * Math.cos(angle1);
            const y1 = cy + maxRadius * Math.sin(angle1);
            const x2 = cx + maxRadius * Math.cos(angle2);
            const y2 = cy + maxRadius * Math.sin(angle2);

            const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
            line.setAttribute('x1', x1);
            line.setAttribute('y1', y1);
            line.setAttribute('x2', x2);
            line.setAttribute('y2', y2);
            svg.appendChild(line);
        }
    }

    /**
     * Export notebook as JSON
     */
    exportJSON() {
        return {
            title: document.querySelector('#notebook-header h1')?.textContent || 'Untitled',
            cells: this.cells.map(c => ({
                index: c.index,
                type: c.type,
                source: c.source,
                output: c.output,
            })),
            timestamp: new Date().toISOString(),
        };
    }

    /**
     * Import notebook from JSON
     */
    importJSON(data) {
        this.cells = [];
        this.elements.cellsContainer.innerHTML = '';

        for (const cellData of data.cells || []) {
            const cell = this.addCell(cellData.type, cellData.index);
            cell.source = cellData.source;
            cell.output = cellData.output;

            // Update DOM
            const textarea = this.elements.cellsContainer.querySelector(
                `[data-index="${cell.index}"] textarea`
            );
            if (textarea) {
                textarea.value = cell.source;
            }

            const outputDiv = this.elements.cellsContainer.querySelector(
                `[data-index="${cell.index}"] .cell-output`
            );
            if (outputDiv && cell.output) {
                outputDiv.textContent = cell.output;
            }
        }

        this.updateStats();
        this.renderDependencyGraph();
    }

    /**
     * Get statistics
     */
    getStats() {
        return {
            cellCount: this.cells.length,
            successCount: this.cells.filter(c => c.status === 'success').length,
            errorCount: this.cells.filter(c => c.status === 'error').length,
            totalTime: this.cells.reduce((s, c) => s + c.executionTime, 0),
        };
    }
}

// Initialize notebook engine on DOM ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.notebookEngine = new NotebookEngine();

        // Add sample cells
        window.notebookEngine.addCell('code');
        window.notebookEngine.addCell('markdown');
        window.notebookEngine.addCell('code');

        // Setup header buttons
        document.getElementById('btn-run-all')?.addEventListener('click', () => {
            window.notebookEngine.executeAll();
        });

        // Setup export receipt button
        document.getElementById('btn-export-receipt')?.addEventListener('click', () => {
            const data = window.notebookEngine.exportJSON();
            const json = JSON.stringify(data, null, 2);
            const blob = new Blob([json], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `notebook-${Date.now()}.json`;
            a.click();
        });

        window.notebookEngine.updateStats();
        window.notebookEngine.renderDependencyGraph();
    });
} else {
    window.notebookEngine = new NotebookEngine();
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = NotebookEngine;
}
