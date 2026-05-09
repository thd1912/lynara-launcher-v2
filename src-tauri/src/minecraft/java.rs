//! Java — detects an installed JRE/JDK or downloads a portable Adoptium JRE.
//!
//! Strategy:
//! 1. Check if the user already has a working Java with the right major version
//!    - `JAVA_HOME` env var
//!    - Common install paths (Adoptium, Microsoft JDK, Zulu, etc. on Windows)
//!    - PATH (`java` command)
//! 2. Otherwise, download a portable JRE from Adoptium API.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::minecraft::{paths, McError, Result};

#[derive(Debug, Clone, Serialize)]
pub struct JavaInstall {
    pub path: PathBuf,
    pub version: u32,
}

/// Tries multiple detection strategies. Returns the first valid Java found.
pub fn detect_system_java(required_major: u32) -> Option<JavaInstall> {
    // Strategy 1: JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let path = PathBuf::from(java_home).join("bin").join(java_executable());
        if let Some(install) = check_java(&path, required_major) {
            return Some(install);
        }
    }

    // Strategy 2: PATH (just `java` in any directory)
    if let Some(install) = check_java(&PathBuf::from("java"), required_major) {
        return Some(install);
    }

    // Strategy 3 (Windows): scan common install paths
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files\Eclipse Adoptium",
            r"C:\Program Files\Java",
            r"C:\Program Files\Microsoft",
            r"C:\Program Files\Zulu",
            r"C:\Program Files\Amazon Corretto",
        ];
        for base in candidates {
            let base = PathBuf::from(base);
            if !base.exists() {
                continue;
            }
            // Iterate on subdirs (each is a Java install)
            if let Ok(entries) = std::fs::read_dir(&base) {
                for entry in entries.flatten() {
                    let java_exe =
                        entry.path().join("bin").join(java_executable());
                    if let Some(install) = check_java(&java_exe, required_major) {
                        return Some(install);
                    }
                }
            }
        }
    }

    // Strategy 4: previously downloaded portable JRE
    if let Ok(runtime_dir) = paths::java_runtime_dir() {
        if runtime_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
                for entry in entries.flatten() {
                    let java_exe =
                        entry.path().join("bin").join(java_executable());
                    if let Some(install) = check_java(&java_exe, required_major) {
                        return Some(install);
                    }
                }
            }
        }
    }

    None
}

fn java_executable() -> &'static str {
    if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    }
}

/// Runs `java -version` and parses the output. Returns the install if valid + matches required version.
fn check_java(path: &Path, required_major: u32) -> Option<JavaInstall> {
    let output = Command::new(path).arg("-version").output().ok()?;

    // `java -version` writes to stderr (yes, really)
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version = parse_java_version(&stderr)?;

    if version >= required_major {
        Some(JavaInstall {
            path: path.to_path_buf(),
            version,
        })
    } else {
        None
    }
}

/// Parses `java -version` output. Examples:
/// - `openjdk version "21.0.1" 2023-10-17`        → 21
/// - `java version "1.8.0_271"`                   → 8 (legacy "1.x" format)
/// - `openjdk version "17.0.7" 2023-04-18`        → 17
fn parse_java_version(output: &str) -> Option<u32> {
    // Find the first `"..."` block
    let start = output.find('"')? + 1;
    let end = output[start..].find('"')? + start;
    let version_str = &output[start..end];

    let parts: Vec<&str> = version_str.split('.').collect();
    let first: u32 = parts.first()?.parse().ok()?;

    // Java 8 and below report "1.8.0_xxx" — the major is the SECOND component
    if first == 1 && parts.len() >= 2 {
        parts[1].parse().ok()
    } else {
        Some(first)
    }
}

// =====================================================================
// ADOPTIUM DOWNLOAD (fallback)
// =====================================================================

const ADOPTIUM_API: &str = "https://api.adoptium.net/v3/assets/latest";

#[derive(Deserialize, Debug)]
struct AdoptiumRelease {
    binary: AdoptiumBinary,
    release_name: String,
}

#[derive(Deserialize, Debug)]
struct AdoptiumBinary {
    package: AdoptiumPackage,
}

#[derive(Deserialize, Debug)]
struct AdoptiumPackage {
    link: String,
    checksum: String,
    name: String,
}

/// Returns an Adoptium download URL for the requested major version.
/// Picks the right OS + architecture automatically.
pub async fn fetch_adoptium_url(major: u32) -> Result<(String, String)> {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    };

    let url = format!(
        "{}/{}/hotspot?architecture={}&image_type=jre&os={}&vendor=eclipse",
        ADOPTIUM_API, major, arch, os
    );

    let resp = reqwest::get(&url).await?;
    if !resp.status().is_success() {
        return Err(McError::Custom(format!(
            "Adoptium HTTP {}",
            resp.status()
        )));
    }

    let releases: Vec<AdoptiumRelease> = resp.json().await?;
    let release = releases
        .first()
        .ok_or_else(|| McError::Custom("Aucune release Adoptium trouvée".into()))?;

    Ok((
        release.binary.package.link.clone(),
        release.binary.package.name.clone(),
    ))
}

// NOTE: actual download + extraction will be in Phase 3.2 with the rest of
// the downloader system. For 3.1 we just expose detection + fetch_adoptium_url.

/// Top-level convenience: get a working Java for the given major version.
/// Tries detection first, returns None if nothing valid found (download will happen in 3.2).
pub fn get_or_detect_java(required_major: u32) -> Option<JavaInstall> {
    detect_system_java(required_major)
}
