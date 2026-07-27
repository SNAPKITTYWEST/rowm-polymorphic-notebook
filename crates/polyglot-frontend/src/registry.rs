//! Language Registry — Central dispatcher for all parsers

use crate::language::{Language, LanguageTier};
use subleq_ir::ast::Program;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use tracing::{debug, info};

/// Parser trait — all languages implement this
pub trait Parser: Send + Sync {
    /// Parse source code into unified AST
    fn parse(&self, source: &str, language: &str) -> Result<Program>;

    /// Language code this parser handles
    fn language_code(&self) -> &'static str;

    /// Quick validation of source syntax
    fn validate(&self, source: &str) -> Result<()>;
}

/// Central registry for all language parsers
pub struct LanguageRegistry {
    languages: HashMap<String, Language>,
    parsers: HashMap<String, Box<dyn Parser>>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        Self {
            languages: HashMap::new(),
            parsers: HashMap::new(),
        }
    }

    /// Initialize with all available languages
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register all languages
        for lang in Language::all_languages() {
            registry.register_language(lang);
        }

        // Register parsers (will be populated by parser implementations)
        // TODO: register actual parser implementations as crate feature gates
        info!("Language registry initialized with {} languages", registry.languages.len());
        registry
    }

    pub fn register_language(&mut self, lang: Language) {
        self.languages.insert(lang.code.clone(), lang);
    }

    pub fn register_parser(&mut self, parser: Box<dyn Parser>) {
        let code = parser.language_code().to_string();
        self.parsers.insert(code, parser);
    }

    /// Parse source code in a given language
    pub fn parse(&self, source: &str, language_code: &str) -> Result<Program> {
        let language = self.languages.get(language_code)
            .ok_or_else(|| anyhow!("Unknown language: {}", language_code))?;

        if !language.enabled {
            return Err(anyhow!("Language {} not enabled", language_code));
        }

        let parser = self.parsers.get(language_code)
            .ok_or_else(|| anyhow!("No parser for language: {}", language_code))?;

        debug!("Parsing {} code ({} chars)", language.name, source.len());
        parser.parse(source, language_code)
    }

    /// Validate source syntax without parsing
    pub fn validate(&self, source: &str, language_code: &str) -> Result<()> {
        let parser = self.parsers.get(language_code)
            .ok_or_else(|| anyhow!("No parser for language: {}", language_code))?;

        parser.validate(source)
    }

    /// Get all supported languages
    pub fn languages(&self) -> Vec<&Language> {
        self.languages.values().collect()
    }

    /// Get enabled languages only
    pub fn enabled_languages(&self) -> Vec<&Language> {
        self.languages.values()
            .filter(|l| l.enabled)
            .collect()
    }

    /// Get languages by tier
    pub fn languages_by_tier(&self, tier: LanguageTier) -> Vec<&Language> {
        self.languages.values()
            .filter(|l| l.tier == tier && l.enabled)
            .collect()
    }

    /// Get language by code
    pub fn get_language(&self, code: &str) -> Option<&Language> {
        self.languages.get(code)
    }

    pub fn has_parser(&self, language_code: &str) -> bool {
        self.parsers.contains_key(language_code)
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockParser;

    impl Parser for MockParser {
        fn parse(&self, _source: &str, _language: &str) -> Result<Program> {
            Err(anyhow!("Mock parser does not parse"))
        }

        fn language_code(&self) -> &'static str {
            "mock"
        }

        fn validate(&self, _source: &str) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = LanguageRegistry::with_defaults();
        assert!(registry.languages.len() >= 17);
    }

    #[test]
    fn test_language_lookup() {
        let registry = LanguageRegistry::with_defaults();
        let py = registry.get_language("py");
        assert!(py.is_some());
        assert_eq!(py.unwrap().name, "Python");
    }

    #[test]
    fn test_register_parser() {
        let mut registry = LanguageRegistry::new();
        registry.register_parser(Box::new(MockParser));
        assert!(registry.has_parser("mock"));
    }

    #[test]
    fn test_languages_by_tier() {
        let registry = LanguageRegistry::with_defaults();
        let tier1 = registry.languages_by_tier(crate::language::LanguageTier::Tier1);
        assert!(tier1.len() >= 8);
    }

    #[test]
    fn test_enabled_languages() {
        let registry = LanguageRegistry::with_defaults();
        let enabled = registry.enabled_languages();
        assert!(enabled.len() > 0);
    }
}
