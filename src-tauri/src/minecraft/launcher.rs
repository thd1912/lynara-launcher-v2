//! Launcher — builds the Java command line, extracts natives, spawns Minecraft.
//! Phase 3.4: supports an optional Fabric profile to launch with Fabric Loader.
//! Phase 3.6: auto-install of mods + shaders + resource packs via Modrinth API.

use serde::Serialize;
use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::auth;
use crate::minecraft::{fabric, java, manifest, paths, McError};

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub server_address: Option<String>,
    pub max_ram_mb: u32,
    pub min_ram_mb: u32,
    /// If set, launch with Fabric Loader instead of vanilla.
    pub fabric_profile: Option<fabric::FabricProfile>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameEvent {
    Started,
    Log { line: String },
    Error { line: String },
    Closed { code: i32 },
}

// =====================================================================
// SESSION RETRIEVAL
// =====================================================================

pub fn load_options_from_session() -> Result<LaunchOptions, McError> {
    let profile = auth::load_session()
        .map_err(|e| McError::Custom(format!("Auth: {}", e)))?
        .ok_or_else(|| McError::Custom("Pas connecté à Microsoft".into()))?;

    Ok(LaunchOptions {
        username: profile.username,
        uuid: profile.uuid,
        access_token: profile.mc_access_token,
        server_address: None,
        max_ram_mb: 4096,
        min_ram_mb: 1024,
        fabric_profile: None,
    })
}

// =====================================================================
// NATIVES EXTRACTION
// =====================================================================

pub fn extract_natives(manifest: &manifest::VersionManifest) -> Result<(), McError> {
    let natives_dir = paths::natives_dir(&manifest.id)?;
    std::fs::create_dir_all(&natives_dir)?;

    let classifier = manifest::current_native_classifier()
        .ok_or_else(|| McError::Custom("Plateforme non supportée".into()))?;

    let libraries_dir = paths::libraries_dir()?;

    for lib in &manifest.libraries {
        if !manifest::is_library_allowed(lib) {
            continue;
        }
        if let Some(downloads) = &lib.downloads {
            if let Some(classifiers) = &downloads.classifiers {
                if let Some(artifact) = classifiers.get(classifier) {
                    let rel = artifact.path.clone().unwrap_or_else(|| {
                        format!(
                            "{}-{}.jar",
                            manifest::maven_path(&lib.name).trim_end_matches(".jar"),
                            classifier
                        )
                    });
                    let jar = libraries_dir.join(&rel);
                    if jar.exists() {
                        extract_jar_natives(&jar, &natives_dir)?;
                    }
                }
            }
        }
        if lib.name.contains(&format!(":{}", classifier)) {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let rel = artifact
                        .path
                        .clone()
                        .unwrap_or_else(|| manifest::maven_path(&lib.name));
                    let jar = libraries_dir.join(&rel);
                    if jar.exists() {
                        extract_jar_natives(&jar, &natives_dir)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn extract_jar_natives(jar_path: &Path, natives_dir: &Path) -> Result<(), McError> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| McError::Custom(format!("Zip {}: {}", jar_path.display(), e)))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| McError::Custom(format!("Zip entry: {}", e)))?;

        let name = entry.name().to_string();
        if name.starts_with("META-INF/") || name.ends_with('/') {
            continue;
        }

        let lower = name.to_lowercase();
        let is_native = lower.ends_with(".dll")
            || lower.ends_with(".so")
            || lower.ends_with(".dylib")
            || lower.ends_with(".jnilib");
        if !is_native {
            continue;
        }

        let flat_name = name.rsplit('/').next().unwrap_or(&name);
        let dest = natives_dir.join(flat_name);

        let mut buffer = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut buffer)
            .map_err(|e| McError::Custom(format!("Read {}: {}", name, e)))?;
        std::fs::write(&dest, buffer)?;
    }

    Ok(())
}

// =====================================================================
// CLASSPATH BUILDER (vanilla + optional Fabric libs)
// =====================================================================

fn build_classpath(
    manifest: &manifest::VersionManifest,
    fabric_profile: Option<&fabric::FabricProfile>,
) -> Result<String, McError> {
    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
    let libraries_dir = paths::libraries_dir()?;
    let mut entries = Vec::new();

    // Add Fabric libraries FIRST so they take precedence
    if let Some(profile) = fabric_profile {
        for lib in &profile.libraries {
            let rel = fabric::maven_path(&lib.name);
            let abs = libraries_dir.join(&rel);
            entries.push(abs.display().to_string());
        }
    }

    // Add vanilla libraries
    for lib in &manifest.libraries {
        if !manifest::is_library_allowed(lib) {
            continue;
        }
        if let Some(downloads) = &lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                let rel = artifact
                    .path
                    .clone()
                    .unwrap_or_else(|| manifest::maven_path(&lib.name));
                let abs = libraries_dir.join(&rel);
                entries.push(abs.display().to_string());
            }
        }
    }

    // Add the version's client.jar at the end
    let client_jar = paths::version_jar(&manifest.id)?;
    entries.push(client_jar.display().to_string());

    Ok(entries.join(separator))
}

// =====================================================================
// LAUNCH
// =====================================================================

pub async fn launch(
    manifest: &manifest::VersionManifest,
    java_install: &java::JavaInstall,
    options: LaunchOptions,
    app: AppHandle,
) -> Result<(), McError> {
    extract_natives(manifest)?;
    let classpath = build_classpath(manifest, options.fabric_profile.as_ref())?;

    let natives_dir = paths::natives_dir(&manifest.id)?;
    let game_dir = paths::root_dir()?;
    let assets_dir = paths::assets_dir()?;

    // Ensure mods directory exists (for Phase 7 custom mod)
    let mods_dir = game_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;

    // Phase 3.6 — Auto-install mods, shaders, and resource packs from Modrinth.
    // Silent + non-blocking : if the API is unreachable we log and continue.
    let mc_version = manifest.id.clone();
    if let Err(e) = crate::assets::ensure_lynara_assets(&game_dir, &mc_version).await {
        eprintln!("[launcher] assets install failed (non-fatal): {}", e);
    }

    let mut cmd = Command::new(&java_install.path);

    // === JVM ARGS ===
    cmd.arg(format!("-Xmx{}M", options.max_ram_mb));
    cmd.arg(format!("-Xms{}M", options.min_ram_mb));
    cmd.arg(format!("-Djava.library.path={}", natives_dir.display()));
    cmd.arg(format!("-Djna.tmpdir={}", natives_dir.display()));
    cmd.arg(format!(
        "-Dorg.lwjgl.system.SharedLibraryExtractPath={}",
        natives_dir.display()
    ));
    cmd.arg(format!("-Dio.netty.native.workdir={}", natives_dir.display()));
    cmd.arg("-Dminecraft.launcher.brand=Lynara");
    cmd.arg("-Dminecraft.launcher.version=2.0");

    cmd.arg("--add-opens=java.base/java.util=ALL-UNNAMED");
    cmd.arg("--add-opens=java.base/java.util.jar=ALL-UNNAMED");
    cmd.arg("--add-opens=java.base/java.lang=ALL-UNNAMED");
    cmd.arg("--add-opens=java.base/java.lang.reflect=ALL-UNNAMED");
    cmd.arg("--add-opens=java.base/java.io=ALL-UNNAMED");
    cmd.arg("--add-opens=java.base/sun.nio.ch=ALL-UNNAMED");

    // Fabric-specific JVM args (from profile)
    if let Some(profile) = &options.fabric_profile {
        if let Some(args) = &profile.arguments {
            for jvm_arg in &args.jvm {
                let resolved = jvm_arg.trim().to_string();
                if !resolved.is_empty() {
                    cmd.arg(resolved);
                }
            }
        }
    }

    cmd.arg("-cp").arg(&classpath);

    // === MAIN CLASS — Fabric's KnotClient if Fabric is enabled, else vanilla
    let main_class = options
        .fabric_profile
        .as_ref()
        .map(|p| p.main_class.clone())
        .unwrap_or_else(|| manifest.main_class.clone());
    cmd.arg(&main_class);

    // === GAME ARGS ===
    cmd.arg("--username").arg(&options.username);
    cmd.arg("--version").arg(&manifest.id);
    cmd.arg("--gameDir").arg(&game_dir);
    cmd.arg("--assetsDir").arg(&assets_dir);
    cmd.arg("--assetIndex").arg(&manifest.asset_index.id);
    cmd.arg("--uuid").arg(&options.uuid);
    cmd.arg("--accessToken").arg(&options.access_token);
    cmd.arg("--clientId").arg("");
    cmd.arg("--xuid").arg("");
    cmd.arg("--userType").arg("msa");
    cmd.arg("--versionType").arg("release");

    if let Some(server) = &options.server_address {
        cmd.arg("--quickPlayMultiplayer").arg(server);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.current_dir(&game_dir);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| McError::Custom(format!("Échec du lancement de Java: {}", e)))?;

    let _ = app.emit("game:event", GameEvent::Started);

    if let Some(stdout) = child.stdout.take() {
        let app = app.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app.emit("game:event", GameEvent::Log { line });
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let app = app.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app.emit("game:event", GameEvent::Error { line });
            }
        });
    }

    let app_close = app.clone();
    tokio::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                let code = status.code().unwrap_or(0);
                let _ = app_close.emit("game:event", GameEvent::Closed { code });
            }
            Err(e) => {
                let _ = app_close.emit(
                    "game:event",
                    GameEvent::Error {
                        line: format!("Erreur d'attente du processus: {}", e),
                    },
                );
            }
        }
    });

    Ok(())
}
