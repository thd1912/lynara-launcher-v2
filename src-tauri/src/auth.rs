//! Auth module — complete Microsoft → Xbox Live → XSTS → Minecraft chain.
//! Tokens are stored in the OS keyring (Windows Credential Manager on Windows).

use keyring::Entry;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Pre-approved client_id used by portablemc — works with the Minecraft API.
/// Mojang doesn't allow new Azure AD apps to call api.minecraftservices.com since 2022,
/// so we reuse this whitelisted ID (this is publicly documented and standard practice
/// for community launchers like Modrinth, Prism, etc.).
const CLIENT_ID: &str = "708e91b5-99f8-4a1d-80ec-e746cbb24771";
const SCOPE: &str = "XboxLive.signin offline_access";

const KEYRING_SERVICE: &str = "fr.lynara.launcher";
const KEYRING_USER: &str = "minecraft_session";

// =====================================================================
// PUBLIC TYPES (serialized to/from frontend)
// =====================================================================

#[derive(Serialize, Clone, Debug)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PollResult {
    AuthorizationPending,
    SlowDown,
    Complete { profile: UserProfile },
    Error { message: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserProfile {
    pub username: String,
    pub uuid: String,
    pub email: Option<String>,
    // Sensitive tokens — kept in struct for backend use, but skipped during JSON serialization
    // when sending to the frontend (so React never sees raw tokens)
    #[serde(skip_serializing)]
    pub mc_access_token: String,
    #[serde(skip_serializing)]
    pub ms_refresh_token: String,
}

// =====================================================================
// MICROSOFT DEVICE CODE FLOW
// =====================================================================

#[derive(Deserialize)]
struct DeviceCodeResponse {
    user_code: String,
    device_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct PollErrorResponse {
    error: String,
}

/// Step 1: Request a device code from Microsoft.
/// Returns the `user_code` shown to the user + the `device_code` used for polling.
pub async fn request_device_code() -> Result<DeviceCodeInfo, String> {
    let client = Client::new();
    let response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPE)])
        .send()
        .await
        .map_err(|e| format!("Erreur réseau: {}", e))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Microsoft a refusé la requête: {}", body));
    }

    let data: DeviceCodeResponse = response
        .json()
        .await
        .map_err(|e| format!("Réponse Microsoft invalide: {}", e))?;

    Ok(DeviceCodeInfo {
        user_code: data.user_code,
        verification_uri: data.verification_uri,
        device_code: data.device_code,
        expires_in: data.expires_in,
        interval: data.interval,
    })
}

/// Step 2: Poll Microsoft for token. Called every `interval` seconds by frontend.
/// Returns the auth state. When `Complete`, the full chain has been run and tokens stored.
pub async fn poll_token(device_code: &str) -> Result<PollResult, String> {
    let client = Client::new();
    let response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&[
            (
                "grant_type",
                "urn:ietf:params:oauth:grant-type:device_code",
            ),
            ("client_id", CLIENT_ID),
            ("device_code", device_code),
        ])
        .send()
        .await
        .map_err(|e| format!("Erreur réseau pendant polling: {}", e))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Lecture réponse échouée: {}", e))?;

    if status.is_success() {
        // Got the Microsoft token! Now run the full chain.
        let token: TokenResponse = serde_json::from_str(&text)
            .map_err(|e| format!("Token Microsoft invalide: {}", e))?;

        match complete_auth_chain(&token.access_token, &token.refresh_token).await {
            Ok(profile) => Ok(PollResult::Complete { profile }),
            Err(e) => Ok(PollResult::Error { message: e }),
        }
    } else {
        // Probably authorization_pending or slow_down — these are normal during polling
        if let Ok(err) = serde_json::from_str::<PollErrorResponse>(&text) {
            match err.error.as_str() {
                "authorization_pending" => Ok(PollResult::AuthorizationPending),
                "slow_down" => Ok(PollResult::SlowDown),
                "expired_token" => Ok(PollResult::Error {
                    message: "Le code a expiré. Recommence la connexion.".into(),
                }),
                "access_denied" => Ok(PollResult::Error {
                    message: "Connexion refusée.".into(),
                }),
                other => Ok(PollResult::Error {
                    message: format!("Erreur Microsoft: {}", other),
                }),
            }
        } else {
            Ok(PollResult::Error {
                message: format!("Réponse inattendue (HTTP {}): {}", status, text),
            })
        }
    }
}

// =====================================================================
// XBOX LIVE + XSTS
// =====================================================================

#[derive(Deserialize)]
struct XboxResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
    xui: Vec<XuiClaim>,
}

#[derive(Deserialize)]
struct XuiClaim {
    uhs: String,
}

/// Step 3: Authenticate with Xbox Live using the Microsoft access token.
/// Returns (xbl_token, uhs).
async fn xbox_live_authenticate(ms_token: &str) -> Result<(String, String), String> {
    let client = Client::new();
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", ms_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let response = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Xbox Live indisponible: {}", e))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Xbox Live a refusé l'authentification: {}", body));
    }

    let data: XboxResponse = response
        .json()
        .await
        .map_err(|e| format!("Réponse Xbox Live invalide: {}", e))?;

    let uhs = data
        .display_claims
        .xui
        .first()
        .ok_or_else(|| "Pas de UHS dans la réponse Xbox".to_string())?
        .uhs
        .clone();

    Ok((data.token, uhs))
}

/// Step 4: Get an XSTS token (the one that authorizes Minecraft API access).
async fn xsts_authenticate(xbl_token: &str) -> Result<String, String> {
    let client = Client::new();
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let response = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("XSTS indisponible: {}", e))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        // Common XSTS errors:
        // - 2148916233 = no Xbox account linked
        // - 2148916235 = not available in country
        // - 2148916238 = under 18 (need adult consent)
        return Err(format!("XSTS a refusé (probablement compte Xbox manquant): {}", body));
    }

    let data: XboxResponse = response
        .json()
        .await
        .map_err(|e| format!("Réponse XSTS invalide: {}", e))?;

    Ok(data.token)
}

// =====================================================================
// MINECRAFT
// =====================================================================

#[derive(Deserialize)]
struct MinecraftAuthResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct MinecraftProfileResponse {
    id: String,
    name: String,
}

/// Step 5: Authenticate with Minecraft API using XSTS token.
async fn minecraft_authenticate(uhs: &str, xsts_token: &str) -> Result<String, String> {
    let client = Client::new();
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", uhs, xsts_token)
    });

    let response = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("API Minecraft indisponible: {}", e))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Minecraft a refusé l'auth: {}", body));
    }

    let data: MinecraftAuthResponse = response
        .json()
        .await
        .map_err(|e| format!("Réponse Minecraft invalide: {}", e))?;

    Ok(data.access_token)
}

/// Step 6: Fetch the Minecraft profile (username + UUID).
/// Will fail with 404 if the account doesn't own Minecraft.
async fn fetch_minecraft_profile(mc_token: &str) -> Result<(String, String), String> {
    let client = Client::new();
    let response = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(mc_token)
        .send()
        .await
        .map_err(|e| format!("Profile indisponible: {}", e))?;

    if response.status() == 404 {
        return Err("Ce compte Microsoft ne possède pas Minecraft Java Edition.".into());
    }
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Récupération du profil échouée: {}", body));
    }

    let profile: MinecraftProfileResponse = response
        .json()
        .await
        .map_err(|e| format!("Profil invalide: {}", e))?;

    Ok((profile.name, profile.id))
}

// =====================================================================
// FULL CHAIN
// =====================================================================

/// Run the entire MS → Xbox → XSTS → MC → Profile chain, then save to keyring.
async fn complete_auth_chain(
    ms_token: &str,
    ms_refresh_token: &str,
) -> Result<UserProfile, String> {
    let (xbl_token, uhs) = xbox_live_authenticate(ms_token).await?;
    let xsts_token = xsts_authenticate(&xbl_token).await?;
    let mc_token = minecraft_authenticate(&uhs, &xsts_token).await?;
    let (username, uuid) = fetch_minecraft_profile(&mc_token).await?;

    let profile = UserProfile {
        username,
        uuid,
        email: None,
        mc_access_token: mc_token,
        ms_refresh_token: ms_refresh_token.to_string(),
    };

    save_session(&profile)?;
    Ok(profile)
}

// =====================================================================
// KEYRING STORAGE (Windows Credential Manager)
// =====================================================================

pub fn save_session(profile: &UserProfile) -> Result<(), String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Keyring init failed: {}", e))?;
    let json = serde_json::to_string(profile)
        .map_err(|e| format!("Serialization failed: {}", e))?;
    // Note: serde_json::to_string respects #[serde(skip_serializing)], so we need
    // a manual save with all fields. Use a separate struct for storage.
    let stored = StoredSession {
        username: profile.username.clone(),
        uuid: profile.uuid.clone(),
        email: profile.email.clone(),
        mc_access_token: profile.mc_access_token.clone(),
        ms_refresh_token: profile.ms_refresh_token.clone(),
    };
    let json_full = serde_json::to_string(&stored)
        .map_err(|e| format!("Serialization failed: {}", e))?;
    let _ = json; // unused (kept for clarity)
    entry
        .set_password(&json_full)
        .map_err(|e| format!("Keyring save failed: {}", e))?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct StoredSession {
    username: String,
    uuid: String,
    email: Option<String>,
    mc_access_token: String,
    ms_refresh_token: String,
}

pub fn load_session() -> Result<Option<UserProfile>, String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Keyring init failed: {}", e))?;
    match entry.get_password() {
        Ok(json) => {
            let stored: StoredSession = serde_json::from_str(&json)
                .map_err(|e| format!("Stored session corrupted: {}", e))?;
            Ok(Some(UserProfile {
                username: stored.username,
                uuid: stored.uuid,
                email: stored.email,
                mc_access_token: stored.mc_access_token,
                ms_refresh_token: stored.ms_refresh_token,
            }))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Keyring load failed: {}", e)),
    }
}

pub fn clear_session() -> Result<(), String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Keyring init failed: {}", e))?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Nothing to clear
        Err(e) => Err(format!("Keyring clear failed: {}", e)),
    }
}
