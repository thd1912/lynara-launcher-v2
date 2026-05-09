//! Paths — centralizes all filesystem locations the launcher uses.
//!
//! Structure (Windows %APPDATA%\.lynara\):
//!
//!   .lynara/
//!     versions/
//!       1.21.4/
//!         1.21.4.json       (version manifest)
//!         1.21.4.jar        (client jar)
//!     libraries/             (Maven-style: net/minecraft/.../foo.jar)
//!     assets/
//!       indexes/<id>.json
//!       objects/<2-char>/<full-hash>
//!     natives/<version>/     (extracted LWJGL DLLs)
//!     java/<runtime>/        (downloaded JRE if needed)
//!     logs/

use std::path::PathBuf;
use crate::minecraft::McError;

const APP_DIRNAME: &str = ".lynara";

/// Root directory: Windows = %APPDATA%\.lynara, Linux = ~/.lynara, macOS = ~/Library/Application Support/.lynara
pub fn root_dir() -> Result<PathBuf, McError> {
    let base = if cfg!(target_os = "windows") {
        dirs::config_dir()
    } else {
        dirs::home_dir()
    }
    .ok_or_else(|| McError::Custom("Impossible de localiser le dossier utilisateur".into()))?;

    Ok(base.join(APP_DIRNAME))
}

pub fn versions_dir() -> Result<PathBuf, McError> {
    Ok(root_dir()?.join("versions"))
}

pub fn version_dir(version_id: &str) -> Result<PathBuf, McError> {
    Ok(versions_dir()?.join(version_id))
}

pub fn version_jar(version_id: &str) -> Result<PathBuf, McError> {
    Ok(version_dir(version_id)?.join(format!("{}.jar", version_id)))
}

pub fn version_json(version_id: &str) -> Result<PathBuf, McError> {
    Ok(version_dir(version_id)?.join(format!("{}.json", version_id)))
}

pub fn libraries_dir() -> Result<PathBuf, McError> {
    Ok(root_dir()?.join("libraries"))
}

pub fn assets_dir() -> Result<PathBuf, McError> {
    Ok(root_dir()?.join("assets"))
}

pub fn assets_indexes_dir() -> Result<PathBuf, McError> {
    Ok(assets_dir()?.join("indexes"))
}

pub fn assets_objects_dir() -> Result<PathBuf, McError> {
    Ok(assets_dir()?.join("objects"))
}

pub fn natives_dir(version_id: &str) -> Result<PathBuf, McError> {
    Ok(root_dir()?.join("natives").join(version_id))
}

pub fn java_runtime_dir() -> Result<PathBuf, McError> {
    Ok(root_dir()?.join("java"))
}

pub fn logs_dir() -> Result<PathBuf, McError> {
    Ok(root_dir()?.join("logs"))
}

/// Ensures the root directory + main subdirectories exist.
/// Idempotent — safe to call multiple times.
pub fn ensure_directories() -> Result<(), McError> {
    for dir in [
        root_dir()?,
        versions_dir()?,
        libraries_dir()?,
        assets_dir()?,
        assets_indexes_dir()?,
        assets_objects_dir()?,
        java_runtime_dir()?,
        logs_dir()?,
    ] {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}
