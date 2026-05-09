//! Installer — orchestrates the full Minecraft installation.
//! Phase 3.4: also handles Fabric loader libraries.

use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::minecraft::{downloader, fabric, manifest, paths, McError};

const CONCURRENCY: usize = 50;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhase {
    Initializing,
    DownloadingClient,
    DownloadingLibraries,
    ExtractingNatives,
    DownloadingAssetIndex,
    DownloadingAssets,
    DownloadingFabric,
    Done,
}

#[derive(Serialize, Clone, Debug)]
pub struct InstallProgress {
    pub phase: InstallPhase,
    pub message: String,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub speed_bps: u64,
    pub files_done: u32,
    pub files_total: u32,
}

#[derive(Deserialize, Debug)]
struct AssetsIndex {
    objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize, Debug)]
struct AssetObject {
    hash: String,
    size: u64,
}

struct InstallState {
    bytes_done: AtomicU64,
    bytes_total: AtomicU64,
    files_done: AtomicU32,
    files_total: AtomicU32,
    phase: Mutex<InstallPhase>,
    message: Mutex<String>,
    speed_window: Mutex<Vec<(std::time::Instant, u64)>>,
}

impl InstallState {
    fn new() -> Self {
        Self {
            bytes_done: AtomicU64::new(0),
            bytes_total: AtomicU64::new(0),
            files_done: AtomicU32::new(0),
            files_total: AtomicU32::new(0),
            phase: Mutex::new(InstallPhase::Initializing),
            message: Mutex::new(String::from("Initialisation...")),
            speed_window: Mutex::new(Vec::new()),
        }
    }

    fn snapshot(&self) -> InstallProgress {
        let now = std::time::Instant::now();
        let bytes_done = self.bytes_done.load(Ordering::Relaxed);

        let speed_bps = {
            let mut win = self.speed_window.lock().unwrap();
            win.push((now, bytes_done));
            win.retain(|(t, _)| now.duration_since(*t).as_secs_f64() < 3.0);
            if win.len() >= 2 {
                let (t0, b0) = win.first().unwrap();
                let dt = now.duration_since(*t0).as_secs_f64();
                if dt > 0.0 {
                    ((bytes_done.saturating_sub(*b0)) as f64 / dt) as u64
                } else {
                    0
                }
            } else {
                0
            }
        };

        InstallProgress {
            phase: self.phase.lock().unwrap().clone(),
            message: self.message.lock().unwrap().clone(),
            bytes_done,
            bytes_total: self.bytes_total.load(Ordering::Relaxed),
            speed_bps,
            files_done: self.files_done.load(Ordering::Relaxed),
            files_total: self.files_total.load(Ordering::Relaxed),
        }
    }

    fn set_phase(&self, phase: InstallPhase, message: String) {
        *self.phase.lock().unwrap() = phase;
        *self.message.lock().unwrap() = message;
    }
}

// =====================================================================
// MAIN ENTRY POINT — installs vanilla + (optionally) Fabric libs
// =====================================================================

pub async fn install_version(
    manifest: manifest::VersionManifest,
    fabric_profile: Option<fabric::FabricProfile>,
    app: AppHandle,
) -> Result<(), McError> {
    paths::ensure_directories()?;

    let state = Arc::new(InstallState::new());
    let client = downloader::http_client();

    let app_for_emit = app.clone();
    let state_for_emit = state.clone();
    let progress_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(200));
        loop {
            interval.tick().await;
            let snapshot = state_for_emit.snapshot();
            let done = matches!(snapshot.phase, InstallPhase::Done);
            let _ = app_for_emit.emit("install:progress", snapshot);
            if done {
                break;
            }
        }
    });

    let (tx, mut rx) = mpsc::unbounded_channel::<u64>();
    let state_for_collector = state.clone();
    let collector_handle = tokio::spawn(async move {
        while let Some(bytes) = rx.recv().await {
            state_for_collector
                .bytes_done
                .fetch_add(bytes, Ordering::Relaxed);
        }
    });

    let result = run_install(&manifest, fabric_profile.as_ref(), &client, &state, tx.clone()).await;

    drop(tx);
    let _ = collector_handle.await;

    state.set_phase(InstallPhase::Done, String::from("Installation terminée"));
    let _ = app.emit("install:progress", state.snapshot());
    progress_handle.abort();

    result
}

async fn run_install(
    manifest: &manifest::VersionManifest,
    fabric_profile: Option<&fabric::FabricProfile>,
    client: &reqwest::Client,
    state: &Arc<InstallState>,
    tx: mpsc::UnboundedSender<u64>,
) -> Result<(), McError> {
    // === PHASE 1 — Client jar ===
    state.set_phase(
        InstallPhase::DownloadingClient,
        String::from("Téléchargement du client Minecraft..."),
    );
    state
        .bytes_total
        .store(manifest.downloads.client.size, Ordering::Relaxed);
    state.files_total.store(1, Ordering::Relaxed);

    let client_jar_path = paths::version_jar(&manifest.id)?;
    downloader::download_file(
        client,
        &manifest.downloads.client.url,
        &client_jar_path,
        Some(manifest.downloads.client.sha1.as_str()),
        &tx,
    )
    .await?;
    state.files_done.fetch_add(1, Ordering::Relaxed);

    // === PHASE 2 — Vanilla libraries ===
    state.set_phase(
        InstallPhase::DownloadingLibraries,
        String::from("Téléchargement des bibliothèques..."),
    );

    let allowed_libs: Vec<&manifest::Library> = manifest
        .libraries
        .iter()
        .filter(|l| manifest::is_library_allowed(l))
        .collect();

    let mut lib_tasks: Vec<(String, PathBuf, String, u64)> = Vec::new();
    for lib in &allowed_libs {
        if let Some(downloads) = &lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                let rel_path = artifact
                    .path
                    .clone()
                    .unwrap_or_else(|| manifest::maven_path(&lib.name));
                let dest = paths::libraries_dir()?.join(&rel_path);
                lib_tasks.push((
                    artifact.url.clone(),
                    dest,
                    artifact.sha1.clone(),
                    artifact.size,
                ));
            }
            if let Some(classifiers) = &downloads.classifiers {
                if let Some(classifier_key) = manifest::current_native_classifier() {
                    if let Some(artifact) = classifiers.get(classifier_key) {
                        let rel_path = artifact.path.clone().unwrap_or_else(|| {
                            format!(
                                "{}-{}.jar",
                                manifest::maven_path(&lib.name).trim_end_matches(".jar"),
                                classifier_key
                            )
                        });
                        let dest = paths::libraries_dir()?.join(&rel_path);
                        lib_tasks.push((
                            artifact.url.clone(),
                            dest,
                            artifact.sha1.clone(),
                            artifact.size,
                        ));
                    }
                }
            }
        }
    }

    let lib_total_bytes: u64 = lib_tasks.iter().map(|(_, _, _, size)| *size).sum();
    let total_so_far =
        manifest.downloads.client.size + lib_total_bytes + manifest.asset_index.size;
    state.bytes_total.store(total_so_far, Ordering::Relaxed);
    state
        .files_total
        .store(1 + lib_tasks.len() as u32 + 1, Ordering::Relaxed);

    let lib_results = stream::iter(lib_tasks)
        .map(|(url, dest, sha1, _)| {
            let client = client.clone();
            let tx = tx.clone();
            let state = state.clone();
            async move {
                let result = downloader::download_file(
                    &client,
                    &url,
                    &dest,
                    Some(sha1.as_str()),
                    &tx,
                )
                .await;
                state.files_done.fetch_add(1, Ordering::Relaxed);
                result
            }
        })
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    for r in lib_results {
        if let Err(e) = r {
            return Err(e);
        }
    }

    // === PHASE 3 — Asset index ===
    state.set_phase(
        InstallPhase::DownloadingAssetIndex,
        String::from("Téléchargement de l'index des ressources..."),
    );
    let asset_index_path =
        paths::assets_indexes_dir()?.join(format!("{}.json", manifest.asset_index.id));
    downloader::download_file(
        client,
        &manifest.asset_index.url,
        &asset_index_path,
        Some(manifest.asset_index.sha1.as_str()),
        &tx,
    )
    .await?;
    state.files_done.fetch_add(1, Ordering::Relaxed);

    // === PHASE 4 — Asset objects ===
    state.set_phase(
        InstallPhase::DownloadingAssets,
        String::from("Téléchargement des ressources (3000+ fichiers)..."),
    );

    let index_bytes = tokio::fs::read(&asset_index_path).await?;
    let asset_index: AssetsIndex = serde_json::from_slice(&index_bytes)?;

    let assets_total_bytes: u64 = asset_index.objects.values().map(|o| o.size).sum();
    state
        .bytes_total
        .fetch_add(assets_total_bytes, Ordering::Relaxed);
    state
        .files_total
        .fetch_add(asset_index.objects.len() as u32, Ordering::Relaxed);

    let assets_objects_dir = paths::assets_objects_dir()?;
    let asset_tasks: Vec<(String, PathBuf, String)> = asset_index
        .objects
        .into_iter()
        .map(|(_name, obj)| {
            let prefix = obj.hash[0..2].to_string();
            let dest = assets_objects_dir.join(&prefix).join(&obj.hash);
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                prefix, obj.hash
            );
            (url, dest, obj.hash)
        })
        .collect();

    let asset_results = stream::iter(asset_tasks)
        .map(|(url, dest, sha1)| {
            let client = client.clone();
            let tx = tx.clone();
            let state = state.clone();
            async move {
                let result = downloader::download_file(
                    &client,
                    &url,
                    &dest,
                    Some(sha1.as_str()),
                    &tx,
                )
                .await;
                state.files_done.fetch_add(1, Ordering::Relaxed);
                result
            }
        })
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    let mut errors = 0;
    for r in asset_results {
        if r.is_err() {
            errors += 1;
            if errors > 10 {
                return Err(McError::Custom(format!(
                    "Trop d'erreurs de téléchargement assets ({})",
                    errors
                )));
            }
        }
    }

    // === PHASE 5 (optional) — Fabric libraries ===
    if let Some(profile) = fabric_profile {
        state.set_phase(
            InstallPhase::DownloadingFabric,
            format!("Téléchargement de Fabric Loader ({})...", profile.id),
        );

        let libraries_dir = paths::libraries_dir()?;

        let fabric_tasks: Vec<(String, PathBuf, Option<String>)> = profile
            .libraries
            .iter()
            .map(|lib| {
                let url = fabric::library_url(lib);
                let dest = libraries_dir.join(fabric::maven_path(&lib.name));
                (url, dest, lib.sha1.clone())
            })
            .collect();

        state
            .files_total
            .fetch_add(fabric_tasks.len() as u32, Ordering::Relaxed);

        let fabric_results = stream::iter(fabric_tasks)
            .map(|(url, dest, sha1)| {
                let client = client.clone();
                let tx = tx.clone();
                let state = state.clone();
                async move {
                    let result = downloader::download_file(
                        &client,
                        &url,
                        &dest,
                        sha1.as_deref(),
                        &tx,
                    )
                    .await;
                    state.files_done.fetch_add(1, Ordering::Relaxed);
                    result
                }
            })
            .buffer_unordered(CONCURRENCY)
            .collect::<Vec<_>>()
            .await;

        for r in fabric_results {
            if let Err(e) = r {
                return Err(McError::Custom(format!("Fabric lib: {}", e)));
            }
        }
    }

    Ok(())
}
