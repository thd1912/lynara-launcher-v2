import { invoke } from "@tauri-apps/api/core";

// =====================================================================
// TYPES (mirror the Rust auth module)
// =====================================================================

export interface DeviceCodeInfo {
  user_code: string;
  verification_uri: string;
  device_code: string;
  expires_in: number;
  interval: number;
}

export interface UserProfile {
  username: string;
  uuid: string;
  email: string | null;
}

export type PollResult =
  | { status: "authorization_pending" }
  | { status: "slow_down" }
  | { status: "complete"; profile: UserProfile }
  | { status: "error"; message: string };

// =====================================================================
// COMMANDS
// =====================================================================

/**
 * Request a Microsoft device code. Returns the user_code to display
 * + the device_code to use for polling.
 */
export async function startDeviceCodeLogin(): Promise<DeviceCodeInfo> {
  return invoke("start_device_code_login");
}

/**
 * Poll Microsoft for completion. Call this every `interval` seconds
 * (provided in DeviceCodeInfo) until you get `complete` or `error`.
 */
export async function pollDeviceCode(
  deviceCode: string
): Promise<PollResult> {
  return invoke("poll_device_code", { deviceCode });
}

/**
 * Check if there's a saved session in the OS keyring.
 * Called at app startup to skip login if user already authenticated.
 */
export async function getCurrentUser(): Promise<UserProfile | null> {
  return invoke("get_current_user");
}

/**
 * Clear the saved session from the OS keyring.
 */
export async function logout(): Promise<void> {
  return invoke("logout");
}
