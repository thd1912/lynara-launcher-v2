//! Minecraft module — pure Rust implementation of a Minecraft launcher backend.

pub mod paths;
pub mod manifest;
pub mod java;
pub mod downloader;
pub mod installer;
pub mod launcher;
pub mod fabric;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum McError {
    #[error("Erreur réseau: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Erreur JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Erreur fichier: {0}")]
    Io(#[from] std::io::Error),

    #[error("Version introuvable: {0}")]
    VersionNotFound(String),

    #[error("Java incompatible: requiert {required}, trouvé {found}")]
    JavaVersionMismatch { required: u32, found: u32 },

    #[error("Java introuvable")]
    JavaNotFound,

    #[error("Erreur: {0}")]
    Custom(String),
}

impl serde::Serialize for McError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, McError>;
