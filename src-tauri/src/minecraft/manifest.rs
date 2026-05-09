//! Manifest — fetches Mojang's version manifest and individual version metadata.
//!
//! Two-tier system:
//! 1. Global manifest (lists all versions): https://piston-meta.mojang.com/mc/game/version_manifest_v2.json
//! 2. Per-version manifest (specific URL from #1): contains libraries, assets, mainClass, etc.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::minecraft::{paths, McError, Result};

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

// =====================================================================
// GLOBAL MANIFEST (list of all versions)
// =====================================================================

#[derive(Deserialize, Debug)]
pub struct GlobalManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Deserialize, Debug)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: u32,
}

/// Fetches the global manifest from Mojang.
pub async fn fetch_global_manifest() -> Result<GlobalManifest> {
    let resp = reqwest::get(VERSION_MANIFEST_URL).await?;
    if !resp.status().is_success() {
        return Err(McError::Custom(format!(
            "Mojang manifest HTTP {}",
            resp.status()
        )));
    }
    let manifest: GlobalManifest = resp.json().await?;
    Ok(manifest)
}

/// Returns the entry for a specific version id (e.g. "1.21.4").
pub fn find_version<'a>(
    manifest: &'a GlobalManifest,
    version_id: &str,
) -> Result<&'a VersionEntry> {
    manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| McError::VersionNotFound(version_id.to_string()))
}

// =====================================================================
// PER-VERSION MANIFEST (the "fat" json with all download info)
// =====================================================================

/// Per-version manifest. Only fields we actually use are typed strictly;
/// everything else is captured in `extra` to remain forward-compatible.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VersionManifest {
    pub id: String,

    #[serde(rename = "type")]
    pub version_type: String,

    /// Required Java major version (e.g. 21 for 1.21+, 17 for 1.17-1.20)
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,

    /// The class containing main() — depends on the loader (vanilla vs Fabric)
    #[serde(rename = "mainClass")]
    pub main_class: String,

    /// Asset index to download
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexRef,

    pub assets: String,

    /// All the JARs to download (libraries)
    pub libraries: Vec<Library>,

    /// Where to fetch the client jar
    pub downloads: ClientDownloads,

    /// Modern format: split between game args and JVM args
    /// Older versions (pre-1.13) use `minecraftArguments` instead
    #[serde(default)]
    pub arguments: Option<Arguments>,

    #[serde(rename = "minecraftArguments", default)]
    pub minecraft_arguments: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    /// OS-conditional include rules
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    /// Native libraries (per-OS classifier mapping)
    #[serde(default)]
    pub natives: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    /// Maps classifier name (e.g. "natives-windows") → Artifact
    #[serde(default)]
    pub classifiers: Option<std::collections::HashMap<String, Artifact>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Artifact {
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Rule {
    pub action: String, // "allow" or "disallow"
    #[serde(default)]
    pub os: Option<OsRule>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClientDownloads {
    pub client: Artifact,
    #[serde(default)]
    pub server: Option<Artifact>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
}

/// Fetches and caches the per-version manifest.
/// If already cached on disk, reads from disk instead of refetching.
pub async fn fetch_version_manifest(entry: &VersionEntry) -> Result<VersionManifest> {
    let cache_path = paths::version_json(&entry.id)?;

    // Read from cache if exists
    if cache_path.exists() {
        let bytes = std::fs::read(&cache_path)?;
        if let Ok(manifest) = serde_json::from_slice::<VersionManifest>(&bytes) {
            return Ok(manifest);
        }
        // If parsing fails, fall through and refetch
    }

    // Fetch from Mojang
    let resp = reqwest::get(&entry.url).await?;
    if !resp.status().is_success() {
        return Err(McError::Custom(format!(
            "Version manifest HTTP {}",
            resp.status()
        )));
    }
    let bytes = resp.bytes().await?;

    // Cache to disk
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cache_path, &bytes)?;

    let manifest: VersionManifest = serde_json::from_slice(&bytes)?;
    Ok(manifest)
}

/// Convenience: fetch global → find version → fetch version manifest.
pub async fn resolve_version(version_id: &str) -> Result<VersionManifest> {
    let global = fetch_global_manifest().await?;
    let entry = find_version(&global, version_id)?;
    fetch_version_manifest(entry).await
}

// =====================================================================
// HELPERS — file size estimation (used for UI progress bar)
// =====================================================================

/// Returns the total byte size of all downloads (libs + client + asset index).
/// Note: doesn't include individual asset objects (those are counted later).
pub fn estimate_initial_download_size(manifest: &VersionManifest) -> u64 {
    let mut total = 0u64;
    total += manifest.downloads.client.size;
    total += manifest.asset_index.size;
    for lib in &manifest.libraries {
        if !is_library_allowed(lib) {
            continue;
        }
        if let Some(downloads) = &lib.downloads {
            if let Some(art) = &downloads.artifact {
                total += art.size;
            }
            if let Some(classifiers) = &downloads.classifiers {
                if let Some(native_classifier) = current_native_classifier() {
                    if let Some(art) = classifiers.get(native_classifier) {
                        total += art.size;
                    }
                }
            }
        }
    }
    total
}

/// Tells whether a library applies to the current OS.
pub fn is_library_allowed(lib: &Library) -> bool {
    let Some(rules) = &lib.rules else { return true };

    let mut allowed = false;
    for rule in rules {
        let matches = rule
            .os
            .as_ref()
            .and_then(|os| os.name.as_ref())
            .map(|name| os_matches(name))
            .unwrap_or(true);

        if matches {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

fn os_matches(name: &str) -> bool {
    match name {
        "windows" => cfg!(target_os = "windows"),
        "osx" | "macos" => cfg!(target_os = "macos"),
        "linux" => cfg!(target_os = "linux"),
        _ => false,
    }
}

/// Returns the natives classifier key for the current OS (e.g. "natives-windows").
pub fn current_native_classifier() -> Option<&'static str> {
    if cfg!(target_os = "windows") {
        Some("natives-windows")
    } else if cfg!(target_os = "macos") {
        Some("natives-macos")
    } else if cfg!(target_os = "linux") {
        Some("natives-linux")
    } else {
        None
    }
}

/// Convert a Maven name like "com.mojang:authlib:5.0.51" to its relative path
/// "com/mojang/authlib/5.0.51/authlib-5.0.51.jar"
pub fn maven_path(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return name.replace(':', "/");
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier_suffix = if parts.len() >= 4 {
        format!("-{}", parts[3])
    } else {
        String::new()
    };
    format!(
        "{}/{}/{}/{}-{}{}.jar",
        group, artifact, version, artifact, version, classifier_suffix
    )
}

/// Checks if a file exists and matches a SHA-1.
pub fn verify_file_sha1(path: &Path, expected_sha1: &str) -> bool {
    use sha1::{Digest, Sha1};

    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    let hex_str = hex::encode(result);
    hex_str.eq_ignore_ascii_case(expected_sha1)
}
