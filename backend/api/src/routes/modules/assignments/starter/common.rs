use serde::Serialize;
use util::{languages::Language};

#[derive(Clone, Debug, Serialize)]
pub struct StarterPack {
    /// Stable ID you’ll send from the UI when picking a pack
    pub id: &'static str,
    /// Human-readable name for dropdowns
    pub name: &'static str,
    /// Language this pack targets
    pub language: Language,
    /// One-line description
    pub description: &'static str,
}

pub fn find_pack(id: &str) -> Option<&'static StarterPack> {
    STARTER_PACKS.iter().find(|p| p.id == id)
}

/// Add your packs here (you can have multiple per language).
pub static STARTER_PACKS: &[StarterPack] = &[
    StarterPack {
        id: "cpp-linkedlist",
        name: "C++ - LinkedList",
        language: Language::Cpp,
        description: "Singly-linked list scaffold (memo/spec/makefile/main).",
    },
    StarterPack {
        id: "java-linkedlist",
        name: "Java - LinkedList",
        language: Language::Java,
        description: "Singly-linked list scaffold (memo/spec/makefile/main).",
    },
        StarterPack {
        id: "python-linkedlist",
        name: "Python - LinkedList",
        language: Language::Python,
        description: "Singly-linked list scaffold (memo/spec/makefile/main).",
    },
    StarterPack {
        id: "rust-linkedlist",
        name: "Rust - LinkedList",
        language: Language::Rust,
        description: "Singly-linked list scaffold (memo/spec/makefile/main).",
    },
    StarterPack {
        id: "go-linkedlist",
        name: "Go - LinkedList",
        language: Language::Go,
        description: "Singly-linked list scaffold (memo/spec/makefile/main).",
    },
    StarterPack {
        id: "c-linkedlist",
        name: "C - LinkedList",
        language: Language::C,
        description: "Singly-linked list scaffold (memo/spec/makefile/main).",
    },
];
