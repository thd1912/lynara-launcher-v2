//! Fabric — fetches Fabric loader metadata and profile from meta.fabricmc.net
//! and provides helpers to build a Fabric-aware launch command.
//!
//! How it works:
//! 1. List of supported Fabric loader versions for a given MC version
//!    GET https://meta.fabricmc.net/v2/versions/loader/<mc_version>
//! 2. Profile JSON for a specific loader version (libraries, mainClass, args)
//!    GET https://meta.fabricmc.net/v2/versions/loader/<mc>/<loader>/profile/json

use serde::Deserialize;

use crate::minecraft::McError;

const FABRIC_META: &str = "https://meta.fabricmc.net/v2";

// =====================================================================
// LOADER VERSIONS LIST
// =====================================================================

#[derive(Deserialize, Debug, Clone)]
pub struct FabricLoaderEntry {
    pub loader: FabricVersionInfo,
    pub intermediary: FabricVersionInfo,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricVersionInfo {
    pub version: String,
    #[serde(default)]
    pub stable: bool,
}

pub async fn fetch_loader_versions(mc_version: &str) -> Result<Vec<FabricLoaderEntry>, McError> {
    let url = format!("{}/versions/loader/{}", FABRIC_META, mc_version);
    let resp = reqwest::get(&url).await?;
    if !resp.status().is_success() {
        return Err(McError::Custom(format!(
            "Fabric meta indisponible (HTTP {}). Soit la version {} n'est pas encore supportée par Fabric, soit le réseau est en panne.",
            resp.status(),
            mc_version
        )));
    }
    let versions: Vec<FabricLoaderEntry> = resp.json().await?;
    Ok(versions)
}

/// Returns the latest stable Fabric loader version for a MC version.
/// Falls back to the latest available (any) if no stable found.
pub async fn latest_loader(mc_version: &str) -> Result<String, McError> {
    let versions = fetch_loader_versions(mc_version).await?;

    if let Some(stable) = versions.iter().find(|v| v.loader.stable) {
        return Ok(stable.loader.version.clone());
    }
    versions
        .first()
        .map(|v| v.loader.version.clone())
        .ok_or_else(|| McError::Custom(format!("Aucune version Fabric pour {}", mc_version)))
}

// =====================================================================
// FABRIC PROFILE JSON (mainClass + libs + args)
// =====================================================================

#[derive(Deserialize, Debug, Clone)]
pub struct FabricProfile {
    pub id: String,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default)]
    pub libraries: Vec<FabricLibrary>,
    #[serde(default)]
    pub arguments: Option<FabricArguments>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricLibrary {
    pub name: String,
    pub url: String,
    /// SHA-1 isn't always present in Fabric meta — verification is best-effort
    #[serde(default)]
    pub sha1: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricArguments {
    #[serde(default)]
    pub game: Vec<String>,
    #[serde(default)]
    pub jvm: Vec<String>,
}

pub async fn fetch_profile(
    mc_version: &str,
    loader_version: &str,
) -> Result<FabricProfile, McError> {
    let url = format!(
        "{}/versions/loader/{}/{}/profile/json",
        FABRIC_META, mc_version, loader_version
    );
    let resp = reqwest::get(&url).await?;
    if !resp.status().is_success() {
        return Err(McError::Custom(format!(
            "Fabric profile HTTP {}",
            resp.status()
        )));
    }
    let profile: FabricProfile = resp.json().await?;
    Ok(profile)
}

/// Convenience: latest stable loader for the given MC version + its profile.
pub async fn resolve_for_version(mc_version: &str) -> Result<FabricProfile, McError> {
    let loader = latest_loader(mc_version).await?;
    fetch_profile(mc_version, &loader).await
}

// =====================================================================
// MAVEN PATH BUILDER
// =====================================================================

/// "net.fabricmc:fabric-loader:0.16.10"
///   → "net/fabricmc/fabric-loader/0.16.10/fabric-loader-0.16.10.jar"
pub fn maven_path(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return name.replace(':', "/");
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    format!(
        "{}/{}/{}/{}-{}.jar",
        group, artifact, version, artifact, version
    )
}

/// Build the full download URL for a Fabric library.
pub fn library_url(lib: &FabricLibrary) -> String {
    let path = maven_path(&lib.name);
    let base = lib.url.trim_end_matches('/');
    format!("{}/{}", base, path)
}
