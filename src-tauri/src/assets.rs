// src-tauri/src/assets.rs
//
// Phase 3.6 — Auto-install of all Lynara assets via Modrinth API.
//
// Manages 3 categories of content :
//   - Mods            (mods/ folder, fabric loader)         : Sodium, Iris
//   - Shader packs    (shaderpacks/ folder, iris loader)    : Complementary Reimagined
//   - Resource packs  (resourcepacks/ folder, no loader)    : Excalibur
//
// Behavior :
//   - Queries Modrinth v2 API for the latest compatible release version.
//   - Downloads silently into the correct folder if not already present.
//   - SHA-1 verification.
//   - Idempotent : skip if already installed (matched by filename prefix).
//   - Auto-update : if a different version is present, replace it.
//   - Fail-safe : if the API is unreachable, log a warning and continue.
//   - For shaders & resource packs : we only DOWNLOAD them.
//     The user activates them manually (Iris menu / Resource Packs menu).

use anyhow::{anyhow, Result};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

const MODRINTH_API_BASE: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "lynara-launcher/1.0 (contact: [email protected])";

#[derive(Debug, Clone, Copy)]
enum AssetKind {
    /// Fabric mod, goes in mods/ folder, filtered by loaders=["fabric"]
    Mod,
    /// Shader pack, goes in shaderpacks/ folder, filtered by loaders=["iris"]
    Shader,
    /// Resource pack, goes in resourcepacks/ folder, filtered by loaders=["minecraft"]
    ResourcePack,
}

impl AssetKind {
    fn folder(&self) -> &'static str {
        match self {
            AssetKind::Mod => "mods",
            AssetKind::Shader => "shaderpacks",
            AssetKind::ResourcePack => "resourcepacks",
        }
    }

    fn loader_filter(&self) -> &'static str {
        match self {
            AssetKind::Mod => "fabric",
            AssetKind::Shader => "iris",
            AssetKind::ResourcePack => "minecraft",
        }
    }
}

/// Definition of an asset to install.
struct AssetDef {
    kind: AssetKind,
    slug: &'static str,
    display_name: &'static str,
    /// Filename prefix used to detect an already-installed version
    /// and to clean up old versions when updating.
    filename_prefix: &'static str,
}

/// Master list of everything Lynara installs automatically.
const ASSETS: &[AssetDef] = &[
    // === Performance mods ===
    AssetDef {
        kind: AssetKind::Mod,
        slug: "sodium",
        display_name: "Sodium",
        filename_prefix: "sodium-fabric",
    },
    AssetDef {
        kind: AssetKind::Mod,
        slug: "iris",
        display_name: "Iris Shaders",
        filename_prefix: "iris-fabric",
    },
    // === Default shader pack (downloaded only, not auto-activated) ===
    AssetDef {
        kind: AssetKind::Shader,
        slug: "complementary-reimagined",
        display_name: "Complementary Reimagined",
        filename_prefix: "ComplementaryReimagined",
    },
    // === Default resource pack (downloaded only, not auto-activated) ===
    AssetDef {
        kind: AssetKind::ResourcePack,
        slug: "excal",
        display_name: "Excalibur",
        filename_prefix: "Excalibur",
    },
];

#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version_number: String,
    files: Vec<ModrinthFile>,
    #[allow(dead_code)]
    #[serde(default)]
    game_versions: Vec<String>,
    #[allow(dead_code)]
    #[serde(default)]
    loaders: Vec<String>,
    #[serde(default = "default_release")]
    version_type: String,
    #[allow(dead_code)]
    #[serde(default)]
    date_published: String,
}

fn default_release() -> String { "release".to_string() }

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    hashes: ModrinthHashes,
    url: String,
    filename: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    size: u64,
}

#[derive(Debug, Deserialize)]
struct ModrinthHashes {
    sha1: String,
}

/// Public entry point. Ensures all configured assets are installed for the given
/// Minecraft version. Never fails the launch: errors are logged and swallowed.
pub async fn ensure_lynara_assets(game_dir: &Path, minecraft_version: &str) -> Result<()> {
    println!("[assets] ensure_lynara_assets start (mc={})", minecraft_version);

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(20))
        .build()?;

    for asset in ASSETS {
        // Make sure the target folder exists
        let folder = game_dir.join(asset.kind.folder());
        if !folder.exists() {
            if let Err(e) = fs::create_dir_all(&folder).await {
                eprintln!("[assets] cannot create {}: {}", folder.display(), e);
                continue;
            }
        }

        match install_one_asset(&client, &folder, minecraft_version, asset).await {
            Ok(InstallStatus::AlreadyPresent(name)) => {
                println!("[assets] {} already present: {}", asset.display_name, name);
            }
            Ok(InstallStatus::Downloaded(name)) => {
                println!("[assets] {} downloaded: {}", asset.display_name, name);
            }
            Ok(InstallStatus::NoVersionAvailable) => {
                eprintln!(
                    "[assets] WARN: no compatible {} version found for MC {} (loader={})",
                    asset.display_name, minecraft_version, asset.kind.loader_filter()
                );
            }
            Err(e) => {
                eprintln!(
                    "[assets] WARN: failed to install {}: {} (continuing)",
                    asset.display_name, e
                );
            }
        }
    }

    println!("[assets] ensure_lynara_assets done");
    Ok(())
}

enum InstallStatus {
    AlreadyPresent(String),
    Downloaded(String),
    NoVersionAvailable,
}

async fn install_one_asset(
    client: &reqwest::Client,
    folder: &Path,
    minecraft_version: &str,
    asset: &AssetDef,
) -> Result<InstallStatus> {
    let loader = asset.kind.loader_filter();

    // Build query URL with proper URL encoding
    let url = format!(
        "{}/project/{}/version?loaders=%5B%22{}%22%5D&game_versions=%5B%22{}%22%5D",
        MODRINTH_API_BASE, asset.slug, loader, minecraft_version
    );
    println!("[assets] {} querying: {}", asset.display_name, url);

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("Modrinth returned {}", resp.status()));
    }

    let versions: Vec<ModrinthVersion> = resp.json().await?;
    if versions.is_empty() {
        return Ok(InstallStatus::NoVersionAvailable);
    }

    // Pick the latest "release" first, fallback to first version
    let chosen = versions
        .iter()
        .find(|v| v.version_type == "release")
        .or_else(|| versions.first())
        .ok_or_else(|| anyhow!("No version available"))?;

    let file = chosen
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| chosen.files.first())
        .ok_or_else(|| anyhow!("No file in chosen version"))?;

    let target_path = folder.join(&file.filename);

    // Check if a version is already installed (any version matching the prefix)
    if let Some(existing) = find_existing_asset(folder, asset.filename_prefix).await? {
        if existing == target_path {
            // Same filename as target -> verify hash
            if verify_sha1(&existing, &file.hashes.sha1).await.unwrap_or(false) {
                return Ok(InstallStatus::AlreadyPresent(file.filename.clone()));
            }
            println!(
                "[assets] {} hash mismatch on existing file, redownloading",
                asset.display_name
            );
        } else {
            // Different version -> replace
            println!(
                "[assets] {} updating: {} -> {}",
                asset.display_name,
                existing.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                file.filename
            );
            let _ = fs::remove_file(&existing).await;
        }
    }

    // Download
    println!(
        "[assets] {} downloading {} ({} bytes)",
        asset.display_name, file.filename, file.size
    );
    download_with_verify(client, &file.url, &target_path, &file.hashes.sha1).await?;

    Ok(InstallStatus::Downloaded(file.filename.clone()))
}

/// Returns the first file in `folder` whose name starts with `prefix` and ends in .jar or .zip
async fn find_existing_asset(folder: &Path, prefix: &str) -> Result<Option<PathBuf>> {
    let mut entries = fs::read_dir(folder).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            // Case-sensitive prefix match (Modrinth filenames are stable)
            if name.starts_with(prefix) {
                let lower = name.to_lowercase();
                if lower.ends_with(".jar") || lower.ends_with(".zip") {
                    return Ok(Some(path));
                }
            }
        }
    }
    Ok(None)
}

async fn download_with_verify(
    client: &reqwest::Client,
    url: &str,
    target_path: &Path,
    expected_sha1: &str,
) -> Result<()> {
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("Download failed: status {}", resp.status()));
    }
    let bytes = resp.bytes().await?;

    let mut file = fs::File::create(target_path).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;
    drop(file);

    if !verify_sha1(target_path, expected_sha1).await.unwrap_or(false) {
        let _ = fs::remove_file(target_path).await;
        return Err(anyhow!("SHA-1 mismatch after download"));
    }
    Ok(())
}

async fn verify_sha1(path: &Path, expected: &str) -> Result<bool> {
    let bytes = fs::read(path).await?;
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let actual = hex::encode(hasher.finalize());
    Ok(actual.eq_ignore_ascii_case(expected))
}
