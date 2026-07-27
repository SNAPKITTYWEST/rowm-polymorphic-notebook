//! Language definitions and tier classification

use serde::{Deserialize, Serialize};

/// Language tiers — organized by maturity and support level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageTier {
    /// Tier 1: Core systems languages (full support)
    Tier1 = 1,
    /// Tier 2: Functional/logic languages (mature support)
    Tier2 = 2,
    /// Tier 3: Array/stack/concatenative (specialized)
    Tier3 = 3,
    /// Tier 4: Esoteric/legacy (experimental)
    Tier4 = 4,
    /// Tier 5: Native substrate (full power)
    Tier5 = 5,
}

/// Language metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub code: String,           // e.g., "py", "js", "rs"
    pub tier: LanguageTier,
    pub file_extensions: Vec<String>,
    pub parser_type: ParserType,
    pub enabled: bool,
}

/// Parser implementation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParserType {
    /// Uses tree-sitter binding
    TreeSitter,
    /// Uses syn or rustc_ast
    Native,
    /// Custom implementation
    Custom,
    /// External command (e.g., M4, native SUBLEQ)
    External,
}

impl Language {
    pub fn python() -> Self {
        Self {
            name: "Python".into(),
            code: "py".into(),
            tier: LanguageTier::Tier1,
            file_extensions: vec!["py".into()],
            parser_type: ParserType::TreeSitter,
            enabled: true,
        }
    }

    pub fn javascript() -> Self {
        Self {
            name: "JavaScript".into(),
            code: "js".into(),
            tier: LanguageTier::Tier1,
            file_extensions: vec!["js".into()],
            parser_type: ParserType::TreeSitter,
            enabled: true,
        }
    }

    pub fn typescript() -> Self {
        Self {
            name: "TypeScript".into(),
            code: "ts".into(),
            tier: LanguageTier::Tier1,
            file_extensions: vec!["ts".into()],
            parser_type: ParserType::TreeSitter,
            enabled: true,
        }
    }

    pub fn rust() -> Self {
        Self {
            name: "Rust".into(),
            code: "rs".into(),
            tier: LanguageTier::Tier1,
            file_extensions: vec!["rs".into()],
            parser_type: ParserType::Native,
            enabled: true,
        }
    }

    pub fn c() -> Self {
        Self {
            name: "C".into(),
            code: "c".into(),
            tier: LanguageTier::Tier1,
            file_extensions: vec!["c".into()],
            parser_type: ParserType::TreeSitter,
            enabled: true,
        }
    }

    pub fn cpp() -> Self {
        Self {
            name: "C++".into(),
            code: "cpp".into(),
            tier: LanguageTier::Tier1,
            file_extensions: vec!["cpp".into(), "cc".into(), "cxx".into()],
            parser_type: ParserType::TreeSitter,
            enabled: true,
        }
    }

    pub fn go() -> Self {
        Self {
            name: "Go".into(),
            code: "go".into(),
            tier: LanguageTier::Tier1,
            file_extensions: vec!["go".into()],
            parser_type: ParserType::TreeSitter,
            enabled: true,
        }
    }

    pub fn zig() -> Self {
        Self {
            name: "Zig".into(),
            code: "zig".into(),
            tier: LanguageTier::Tier1,
            file_extensions: vec!["zig".into()],
            parser_type: ParserType::Custom,
            enabled: false, // TODO: implement Zig parser
        }
    }

    pub fn lisp() -> Self {
        Self {
            name: "Lisp".into(),
            code: "lisp".into(),
            tier: LanguageTier::Tier2,
            file_extensions: vec!["lisp".into(), "cl".into()],
            parser_type: ParserType::Custom,
            enabled: true,
        }
    }

    pub fn scheme() -> Self {
        Self {
            name: "Scheme".into(),
            code: "scm".into(),
            tier: LanguageTier::Tier2,
            file_extensions: vec!["scm".into()],
            parser_type: ParserType::Custom,
            enabled: true,
        }
    }

    pub fn haskell() -> Self {
        Self {
            name: "Haskell".into(),
            code: "hs".into(),
            tier: LanguageTier::Tier2,
            file_extensions: vec!["hs".into()],
            parser_type: ParserType::Custom,
            enabled: true,
        }
    }

    pub fn ocaml() -> Self {
        Self {
            name: "OCaml".into(),
            code: "ml".into(),
            tier: LanguageTier::Tier2,
            file_extensions: vec!["ml".into()],
            parser_type: ParserType::Custom,
            enabled: false, // TODO: implement OCaml parser
        }
    }

    pub fn prolog() -> Self {
        Self {
            name: "Prolog".into(),
            code: "pl".into(),
            tier: LanguageTier::Tier2,
            file_extensions: vec!["pl".into()],
            parser_type: ParserType::Custom,
            enabled: true,
        }
    }

    pub fn apl() -> Self {
        Self {
            name: "APL".into(),
            code: "apl".into(),
            tier: LanguageTier::Tier3,
            file_extensions: vec!["apl".into()],
            parser_type: ParserType::Custom,
            enabled: false, // TODO: implement APL parser
        }
    }

    pub fn forth() -> Self {
        Self {
            name: "Forth".into(),
            code: "fth".into(),
            tier: LanguageTier::Tier3,
            file_extensions: vec!["fth".into()],
            parser_type: ParserType::Custom,
            enabled: true,
        }
    }

    pub fn subleq() -> Self {
        Self {
            name: "SUBLEQ".into(),
            code: "subleq".into(),
            tier: LanguageTier::Tier5,
            file_extensions: vec!["subleq".into()],
            parser_type: ParserType::Custom,
            enabled: true,
        }
    }

    pub fn m4() -> Self {
        Self {
            name: "M4".into(),
            code: "m4".into(),
            tier: LanguageTier::Tier5,
            file_extensions: vec!["m4".into()],
            parser_type: ParserType::External,
            enabled: true,
        }
    }

    pub fn all_languages() -> Vec<Language> {
        vec![
            // Tier 1
            Self::python(),
            Self::javascript(),
            Self::typescript(),
            Self::rust(),
            Self::c(),
            Self::cpp(),
            Self::go(),
            Self::zig(),
            // Tier 2
            Self::lisp(),
            Self::scheme(),
            Self::haskell(),
            Self::ocaml(),
            Self::prolog(),
            // Tier 3
            Self::apl(),
            Self::forth(),
            // Tier 5
            Self::subleq(),
            Self::m4(),
        ]
    }

    pub fn by_tier(tier: LanguageTier) -> Vec<Language> {
        Self::all_languages()
            .into_iter()
            .filter(|l| l.tier == tier)
            .collect()
    }

    pub fn enabled_languages() -> Vec<Language> {
        Self::all_languages()
            .into_iter()
            .filter(|l| l.enabled)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_creation() {
        let py = Language::python();
        assert_eq!(py.code, "py");
        assert_eq!(py.tier, LanguageTier::Tier1);
    }

    #[test]
    fn test_all_languages() {
        let all = Language::all_languages();
        assert!(all.len() >= 17); // At least all defined languages
    }

    #[test]
    fn test_languages_by_tier() {
        let tier1 = Language::by_tier(LanguageTier::Tier1);
        assert!(tier1.len() >= 8);

        let tier5 = Language::by_tier(LanguageTier::Tier5);
        assert!(tier5.len() >= 2);
    }
}
