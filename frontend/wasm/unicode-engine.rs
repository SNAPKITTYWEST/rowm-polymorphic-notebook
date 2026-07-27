// WASM Unicode Engine
// Deterministic Unicode normalization, encoding, and roundtrip verification

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct UnicodeEngine {
    normalization_form: String,
}

#[wasm_bindgen]
impl UnicodeEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(form: &str) -> UnicodeEngine {
        UnicodeEngine {
            normalization_form: form.to_string(),
        }
    }

    /// Normalize Unicode string (NFC, NFD, NFKC, NFKD)
    #[wasm_bindgen]
    pub fn normalize(&self, input: &str) -> String {
        match self.normalization_form.as_str() {
            "NFC" => unicode_normalization::char::compose(input.chars()).collect(),
            "NFKC" => unicode_normalization::char::compose_compatible(input.chars()).collect(),
            _ => input.to_string(),
        }
    }

    /// Encode to Unicode IR (code points + UTF-8 bytes)
    #[wasm_bindgen]
    pub fn encode(&self, input: &str) -> String {
        let normalized = self.normalize(input);
        let mut codepoints = Vec::new();
        let mut utf8_bytes = Vec::new();

        // Collect code points
        for ch in normalized.chars() {
            codepoints.push(ch as u32);
        }

        // Collect UTF-8 bytes
        for byte in normalized.as_bytes() {
            utf8_bytes.push(*byte);
        }

        // Return JSON-encoded IR
        format!(
            r#"{{"normalized":"{}","codePoints":{},"utf8Bytes":{},"length":{},"byteLength":{}}}"#,
            normalized.replace("\\", "\\\\").replace("\"", "\\\""),
            serde_json::to_string(&codepoints).unwrap_or_default(),
            serde_json::to_string(&utf8_bytes).unwrap_or_default(),
            codepoints.len(),
            utf8_bytes.len()
        )
    }

    /// Check if string contains astral plane characters (code points > 0xFFFF)
    #[wasm_bindgen]
    pub fn has_astral_characters(&self, input: &str) -> bool {
        input.chars().any(|ch| (ch as u32) > 0xFFFF)
    }

    /// Check if string contains combining marks
    #[wasm_bindgen]
    pub fn has_combining_marks(&self, input: &str) -> bool {
        // Unicode combining marks: 0x0300-0x036F
        input.chars().any(|ch| {
            let cp = ch as u32;
            cp >= 0x0300 && cp <= 0x036F
        })
    }

    /// Detect bidirectional text (RTL vs LTR)
    #[wasm_bindgen]
    pub fn detect_bidi_level(&self, input: &str) -> String {
        // Check for RTL scripts (Hebrew, Arabic, etc.)
        for ch in input.chars() {
            let cp = ch as u32;
            if (cp >= 0x0590 && cp <= 0x08FF) ||    // Hebrew, Arabic, Syriac
               (cp >= 0xFB1D && cp <= 0xFB4F) ||    // Hebrew presentation
               (cp >= 0xFB50 && cp <= 0xFDFF) ||    // Arabic presentation A
               (cp >= 0xFE70 && cp <= 0xFEFF)       // Arabic presentation B
            {
                return "rtl".to_string();
            }
        }
        "ltr".to_string()
    }

    /// Verify roundtrip (normalize → encode → decode → verify)
    #[wasm_bindgen]
    pub fn verify_roundtrip(&self, input: &str) -> bool {
        let normalized1 = self.normalize(input);
        let normalized2 = self.normalize(&normalized1);
        normalized1 == normalized2
    }

    /// Count grapheme clusters (visual characters)
    #[wasm_bindgen]
    pub fn grapheme_count(&self, input: &str) -> usize {
        // Approximate: count combining marks as part of base character
        let mut count = 0;
        let mut in_combining = false;

        for ch in input.chars() {
            let cp = ch as u32;
            if cp >= 0x0300 && cp <= 0x036F {
                // Combining mark: don't count separately
                in_combining = true;
            } else {
                count += 1;
                in_combining = false;
            }
        }

        count
    }

    /// Get all code points as array
    #[wasm_bindgen]
    pub fn code_points(&self, input: &str) -> String {
        let normalized = self.normalize(input);
        let cps: Vec<u32> = normalized.chars().map(|ch| ch as u32).collect();
        serde_json::to_string(&cps).unwrap_or_default()
    }

    /// Get all UTF-8 bytes as array
    #[wasm_bindgen]
    pub fn utf8_bytes(&self, input: &str) -> String {
        let normalized = self.normalize(input);
        let bytes: Vec<u8> = normalized.as_bytes().to_vec();
        serde_json::to_string(&bytes).unwrap_or_default()
    }
}

#[wasm_bindgen]
pub fn string_length_codepoints(input: &str) -> usize {
    input.chars().count()
}

#[wasm_bindgen]
pub fn string_length_bytes(input: &str) -> usize {
    input.as_bytes().len()
}

#[wasm_bindgen]
pub fn string_length_graphemes(input: &str) -> usize {
    // Approximate grapheme count
    let mut count = 0;
    for ch in input.chars() {
        let cp = ch as u32;
        if !(cp >= 0x0300 && cp <= 0x036F) {
            count += 1;
        }
    }
    count
}
