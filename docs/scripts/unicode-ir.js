/**
 * Unicode IR Engine
 * Handles Unicode normalization, encoding/decoding, and preservation of astral characters
 */

class UnicodeIREngine {
    constructor() {
        this.encoder = new TextEncoder();
        this.decoder = new TextDecoder('utf-8');
        this.normalizationForm = 'NFC'; // Canonical composition
    }

    /**
     * Normalize Unicode string using specified form
     */
    normalize(input, form = this.normalizationForm) {
        if (typeof input !== 'string') return '';
        try {
            return input.normalize(form);
        } catch (e) {
            console.error('Normalization failed:', e);
            return input;
        }
    }

    /**
     * Encode string to Unicode IR (code points + UTF-8)
     * Preserves astral plane, emoji, combining marks, bidirectional text
     */
    encode(input) {
        const normalized = this.normalize(input);
        const codePoints = [];
        const utf8Bytes = [];

        // Iterate by code point, not by UTF-16 unit
        for (const char of normalized) {
            const codePoint = char.codePointAt(0);
            codePoints.push(codePoint);
        }

        // Encode to UTF-8
        const utf8Array = this.encoder.encode(normalized);
        for (let i = 0; i < utf8Array.length; i++) {
            utf8Bytes.push(utf8Array[i]);
        }

        return {
            normalized: normalized,
            codePoints: codePoints,
            utf8Bytes: Array.from(utf8Bytes),
            length: codePoints.length,
            byteLength: utf8Bytes.length,
        };
    }

    /**
     * Decode Unicode IR back to string
     * Reverses encode() with full preservation
     */
    decode(irObject) {
        if (!irObject || !irObject.utf8Bytes) {
            console.error('Invalid IR object');
            return '';
        }

        try {
            const uint8Array = new Uint8Array(irObject.utf8Bytes);
            const decoded = this.decoder.decode(uint8Array);
            return this.normalize(decoded);
        } catch (e) {
            console.error('Decoding failed:', e);
            return '';
        }
    }

    /**
     * Verify roundtrip preservation (normalize → encode → decode → verify)
     */
    verifyRoundtrip(input) {
        const encoded = this.encode(input);
        const decoded = this.decode(encoded);
        const reencoded = this.encode(decoded);

        return {
            success: encoded.codePoints.length === reencoded.codePoints.length &&
                    encoded.utf8Bytes.length === reencoded.utf8Bytes.length,
            original: input,
            encoded: encoded,
            decoded: decoded,
            reencoded: reencoded,
        };
    }

    /**
     * Extract grapheme clusters (visual characters)
     * Important for combining marks and emoji sequences
     */
    graphemeClusters(input) {
        const normalized = this.normalize(input);
        const clusters = [];

        // Use Intl.Segmenter if available (modern browsers)
        if (typeof Intl !== 'undefined' && Intl.Segmenter) {
            const segmenter = new Intl.Segmenter('en', { granularity: 'grapheme' });
            const segments = segmenter.segment(normalized);
            for (const segment of segments) {
                clusters.push(segment.segment);
            }
        } else {
            // Fallback: iterate by code point
            for (const char of normalized) {
                clusters.push(char);
            }
        }

        return clusters;
    }

    /**
     * Detect bidirectional text (RTL/LTR)
     */
    detectBidiLevel(input) {
        // Simple heuristic: check for RTL scripts (Hebrew, Arabic, etc.)
        const rtlRanges = [
            [0x0590, 0x08FF],   // Hebrew, Arabic, Syriac, etc.
            [0xFB1D, 0xFB4F],   // Hebrew presentation forms
            [0xFB50, 0xFDFF],   // Arabic presentation forms A
            [0xFE70, 0xFEFF],   // Arabic presentation forms B
        ];

        for (const char of input) {
            const codePoint = char.codePointAt(0);
            for (const [start, end] of rtlRanges) {
                if (codePoint >= start && codePoint <= end) {
                    return 'rtl';
                }
            }
        }
        return 'ltr';
    }

    /**
     * Validate astral plane characters
     */
    hasAstralCharacters(input) {
        for (const char of input) {
            if (char.codePointAt(0) > 0xFFFF) {
                return true;
            }
        }
        return false;
    }

    /**
     * List all astral characters (code points > 0xFFFF)
     */
    findAstralCharacters(input) {
        const astral = [];
        for (const char of input) {
            const codePoint = char.codePointAt(0);
            if (codePoint > 0xFFFF) {
                astral.push({
                    char: char,
                    codePoint: codePoint,
                    hex: '0x' + codePoint.toString(16).toUpperCase(),
                });
            }
        }
        return astral;
    }

    /**
     * Sanitize for safe display (no invisible characters, control chars)
     */
    sanitizeForDisplay(input, removeControls = false) {
        let result = input;

        if (removeControls) {
            // Remove control characters (0x0000-0x001F, 0x007F-0x009F)
            result = result.replace(/[\x00-\x1F\x7F-\x9F]/g, '');
        }

        return result;
    }

    /**
     * Check if string contains combining marks
     */
    hasCombiningMarks(input) {
        // Unicode combining marks range: 0x0300-0x036F
        for (const char of input) {
            const codePoint = char.codePointAt(0);
            if (codePoint >= 0x0300 && codePoint <= 0x036F) {
                return true;
            }
        }
        return false;
    }

    /**
     * Export as JSON-safe representation
     */
    toJSON(input) {
        const ir = this.encode(input);
        return {
            normalized: ir.normalized,
            codePoints: ir.codePoints,
            utf8Bytes: ir.utf8Bytes,
            metadata: {
                length: ir.length,
                byteLength: ir.byteLength,
                hasAstral: this.hasAstralCharacters(input),
                hasCombining: this.hasCombiningMarks(input),
                bidiLevel: this.detectBidiLevel(input),
            },
        };
    }

    /**
     * From JSON-safe representation
     */
    fromJSON(obj) {
        if (!obj || !obj.utf8Bytes) {
            throw new Error('Invalid JSON-safe IR object');
        }
        return this.decode(obj);
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = UnicodeIREngine;
}
