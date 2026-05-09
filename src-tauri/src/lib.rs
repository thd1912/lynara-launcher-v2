// Lynara Launcher v2 - Tauri backend entry point — Phase 3.4 with Fabric

mod auth;
mod minecraft;
mod assets;

use auth::{DeviceCodeInfo, PollResult, UserProfile};
use minecraft::manifest::VersionManifest;
use minecraft::java::JavaInstall;
use minecraft::launcher::LaunchOptions;
use std::sync::Mutex;

#[derive(Default)]
struct AppState {
    session: Mutex<Option<LaunchOptions>>,
}

fn populate_session(state: &tauri::State<AppState>, profile: &UserProfile) {
    let options = LaunchOptions {
        username: profile.username.clone(),
        uuid: profile.uuid.clone(),
        access_token: profile.mc_access_token.clone(),
        server_address: None,
        max_ram_mb: 4096,
        min_ram_mb: 1024,
        fabric_profile: None,
    };
    *state.session.lock().unwrap() = Some(options);
}

// =====================================================================
// AUTH
// =====================================================================

#[tauri::command]
async fn start_device_code_login() -> Result<DeviceCodeInfo, String> {
    auth::request_device_code().await
}

#[tauri::command]
async fn poll_device_code(
    device_code: String,
    state: tauri::State<'_, AppState>,
) -> Result<PollResult, String> {
    let result = auth::poll_token(&device_code).await?;
    if let PollResult::Complete { profile } = &result {
        populate_session(&state, profile);
    }
    Ok(result)
}

#[tauri::command]
fn get_current_user(
    state: tauri::State<'_, AppState>,
) -> Result<Option<UserProfile>, String> {
    let result = auth::load_session();
    if let Ok(Some(profile)) = &result {
        populate_session(&state, profile);
    }
    result
}

#[tauri::command]
fn logout(state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.session.lock().unwrap() = None;
    auth::clear_session()
}

// =====================================================================
// MINECRAFT
// =====================================================================

#[tauri::command]
async fn prepare_minecraft(version_id: String) -> Result<PrepareResult, minecraft::McError> {
    minecraft::paths::ensure_directories()?;
    let manifest = minecraft::manifest::resolve_version(&version_id).await?;
    let required_major = manifest.java_version.major_version;
    let java = minecraft::java::get_or_detect_java(required_major);
    let estimated_size = minecraft::manifest::estimate_initial_download_size(&manifest);

    Ok(PrepareResult {
        version_id: manifest.id.clone(),
        java_required: required_major,
        java_found: java,
        main_class: manifest.main_class.clone(),
        estimated_download_bytes: estimated_size,
        manifest,
    })
}

#[derive(serde::Serialize)]
struct PrepareResult {
    version_id: String,
    java_required: u32,
    java_found: Option<JavaInstall>,
    main_class: String,
    estimated_download_bytes: u64,
    manifest: VersionManifest,
}

#[tauri::command]
async fn install_minecraft(
    version_id: String,
    app: tauri::AppHandle,
) -> Result<(), minecraft::McError> {
    let manifest = minecraft::manifest::resolve_version(&version_id).await?;

    // Phase 3.4: also fetch Fabric profile so we can install its libs alongside
    let fabric_profile = match minecraft::fabric::resolve_for_version(&version_id).await {
        Ok(profile) => Some(profile),
        Err(e) => {
            // Fabric isn't required for the install to succeed (can launch vanilla too)
            eprintln!("[lynara] Fabric meta unavailable: {}", e);
            None
        }
    };

    minecraft::installer::install_version(manifest, fabric_profile, app).await
}

#[tauri::command]
async fn launch_minecraft(
    version_id: String,
    server_address: Option<String>,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), minecraft::McError> {
    let mut options = {
        let guard = state.session.lock().unwrap();
        guard.as_ref().cloned().ok_or_else(|| {
            minecraft::McError::Custom(
                "Pas de session active. Clique 'Déconnexion' puis reconnecte-toi.".into(),
            )
        })?
    };
    options.server_address = server_address;

    let manifest = minecraft::manifest::resolve_version(&version_id).await?;
    let java = minecraft::java::get_or_detect_java(manifest.java_version.major_version)
        .ok_or(minecraft::McError::JavaNotFound)?;

    // Phase 3.4: fetch Fabric profile and use it for launch
    let fabric_profile = match minecraft::fabric::resolve_for_version(&version_id).await {
        Ok(profile) => Some(profile),
        Err(e) => {
            eprintln!("[lynara] Fabric meta unavailable, launching vanilla: {}", e);
            None
        }
    };
    options.fabric_profile = fabric_profile;

    minecraft::launcher::launch(&manifest, &java, options, app).await
}

// =====================================================================
// ENTRY POINT
// =====================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_device_code_login,
            poll_device_code,
            get_current_user,
            logout,
            prepare_minecraft,
            install_minecraft,
            launch_minecraft,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
