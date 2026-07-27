/**
 * WORM Receipt Chain
 * Generates deterministic, cryptographically sealed execution receipts
 */

class WORMReceiptChain {
    constructor() {
        this.receipts = [];
        this.chain = [];
        this.previousHash = this.genesisHash();
    }

    /**
     * Genesis hash (all zeros for first receipt)
     */
    genesisHash() {
        return '0'.repeat(128); // SHA-512 equivalent (128 hex chars)
    }

    /**
     * Generate deterministic canonical form (excludes timestamp)
     */
    canonicalForm(receiptData) {
        const {
            sequenceNumber,
            agentId,
            capabilityId,
            instructionHash,
            action,
            inputHash,
            outputHash,
            keyVersion,
            signature,
            status,
        } = receiptData;

        // Deterministic field order (excludes timestamp)
        return [
            `seq:${sequenceNumber}`,
            `agent:${agentId}`,
            `cap:${capabilityId}`,
            `instr:${instructionHash}`,
            `action:${action}`,
            `input:${inputHash}`,
            `output:${outputHash}`,
            `keyver:${keyVersion}`,
            `sig:${signature}`,
            `status:${status}`,
        ].join('|');
    }

    /**
     * Compute SHA-512 hash (using Web Crypto)
     */
    async computeSHA512(data) {
        const encoder = new TextEncoder();
        const dataBuffer = encoder.encode(data);
        const hashBuffer = await crypto.subtle.digest('SHA-512', dataBuffer);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    }

    /**
     * Create a new receipt
     */
    async createReceipt(receiptData) {
        const sequenceNumber = this.receipts.length + 1;
        const canonical = this.canonicalForm({
            sequenceNumber,
            ...receiptData,
        });

        const receiptHash = await this.computeSHA512(canonical);

        const receipt = {
            sequenceNumber,
            receiptId: `rcpt-${sequenceNumber.toString().padStart(10, '0')}`,
            receiptHash,
            previousHash: this.previousHash,
            canonical,
            timestamp: Math.floor(Date.now() / 1000),
            ...receiptData,
        };

        this.receipts.push(receipt);
        this.previousHash = receiptHash;
        this.chain.push({
            sequence: sequenceNumber,
            hash: receiptHash,
            previousHash: receipt.previousHash,
        });

        return receipt;
    }

    /**
     * Verify chain integrity
     */
    verifyChainIntegrity() {
        for (let i = 0; i < this.receipts.length; i++) {
            const receipt = this.receipts[i];

            // Verify hash is deterministic
            const canonical = this.canonicalForm({
                sequenceNumber: receipt.sequenceNumber,
                agentId: receipt.agentId,
                capabilityId: receipt.capabilityId,
                instructionHash: receipt.instructionHash,
                action: receipt.action,
                inputHash: receipt.inputHash,
                outputHash: receipt.outputHash,
                keyVersion: receipt.keyVersion,
                signature: receipt.signature,
                status: receipt.status,
            });

            if (canonical !== receipt.canonical) {
                return {
                    valid: false,
                    error: `Receipt ${i}: canonical form mismatch`,
                    receipt: receipt,
                };
            }

            // Verify chain linkage
            if (i > 0) {
                const prevReceipt = this.receipts[i - 1];
                if (receipt.previousHash !== prevReceipt.receiptHash) {
                    return {
                        valid: false,
                        error: `Receipt ${i}: previous hash mismatch`,
                        receipt: receipt,
                    };
                }
            } else {
                // First receipt should link to genesis
                if (receipt.previousHash !== this.genesisHash()) {
                    return {
                        valid: false,
                        error: `Receipt 0: should link to genesis hash`,
                        receipt: receipt,
                    };
                }
            }
        }

        return { valid: true, receiptsVerified: this.receipts.length };
    }

    /**
     * Export receipt as JSON
     */
    exportReceipt(index) {
        if (index < 0 || index >= this.receipts.length) {
            throw new Error('Receipt index out of bounds');
        }

        const receipt = this.receipts[index];
        return {
            receiptId: receipt.receiptId,
            sequenceNumber: receipt.sequenceNumber,
            receiptHash: receipt.receiptHash,
            previousHash: receipt.previousHash,
            agentId: receipt.agentId,
            capabilityId: receipt.capabilityId,
            action: receipt.action,
            status: receipt.status,
            canonical: receipt.canonical,
            timestamp: receipt.timestamp,
        };
    }

    /**
     * Export entire chain as JSON
     */
    exportChain() {
        return {
            totalReceipts: this.receipts.length,
            genesisHash: this.genesisHash(),
            receipts: this.receipts.map((r, i) => this.exportReceipt(i)),
            chainValid: this.verifyChainIntegrity().valid,
        };
    }

    /**
     * Get receipt by sequence number
     */
    getReceipt(sequenceNumber) {
        return this.receipts.find(r => r.sequenceNumber === sequenceNumber);
    }

    /**
     * Get last receipt
     */
    getLastReceipt() {
        return this.receipts[this.receipts.length - 1];
    }

    /**
     * Clear all receipts
     */
    clear() {
        this.receipts = [];
        this.chain = [];
        this.previousHash = this.genesisHash();
    }

    /**
     * Get chain statistics
     */
    getStats() {
        return {
            totalReceipts: this.receipts.length,
            chainValid: this.verifyChainIntegrity().valid,
            genesisHash: this.genesisHash(),
            currentHeadHash: this.previousHash,
            receipts: this.receipts.map(r => ({
                sequence: r.sequenceNumber,
                receiptId: r.receiptId,
                hash: r.receiptHash,
            })),
        };
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = WORMReceiptChain;
}
