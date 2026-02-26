import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { DownloadTask, DownloadState } from "$lib/types/models";

export async function getDownloads(): Promise<DownloadTask[]> {
	return invoke<DownloadTask[]>("get_downloads");
}

export async function addDownload(url: string, save_path: string): Promise<DownloadTask> {
	return invoke<DownloadTask>("add_download", { url, savePath: save_path });
}

export async function removeDownload(id: string): Promise<void> {
	return invoke<void>("remove_download", { id });
}

export async function pauseDownload(id: string): Promise<void> {
	return invoke<void>("pause_download", { id });
}

export async function resumeDownload(id: string): Promise<void> {
	return invoke<void>("resume_download", { id });
}

export async function cancelDownload(id: string): Promise<void> {
	return invoke<void>("cancel_download", { id });
}

export async function moveDownload(id: string, direction: "up" | "down"): Promise<void> {
	return invoke<void>("move_download", { id, direction });
}

export async function forceStartDownload(id: string): Promise<void> {
	return invoke<void>("force_start", { id });
}

export async function verifyChecksum(
	id: string,
	hashType: "md5" | "sha256",
	expectedHash: string,
): Promise<boolean> {
	return invoke<boolean>("verify_checksum", { id, hashType, expectedHash });
}

export interface DownloadProgressPayload {
	id: string;
	state: DownloadState;
	downloaded_bytes: number;
	total_bytes: number;
}

export function listenToProgress(
	callback: (payload: DownloadProgressPayload) => void,
): Promise<() => void> {
	return listen<DownloadProgressPayload>("download_progress", (event) => {
		callback(event.payload);
	});
}

export function listenToClipboardUrl(
	callback: (url: string) => void,
): Promise<() => void> {
	return listen<string>("clipboard_url_detected", (event) => {
		callback(event.payload);
	});
}

export function listenToDownloadAdded(
	callback: (task: DownloadTask) => void,
): Promise<() => void> {
	return listen<DownloadTask>("download_added", (event) => {
		callback(event.payload);
	});
}

export async function setGlobalSpeedLimit(bytes_per_sec: number): Promise<void> {
	return invoke<void>("set_global_speed_limit", { bytesPerSec: bytes_per_sec });
}

export async function setDownloadSpeedLimit(id: string, bytes_per_sec: number | null): Promise<void> {
	return invoke<void>("set_download_speed_limit", { id, bytesPerSec: bytes_per_sec });
}

export async function setScheduleConfig(enabled: boolean, startTime: string, endTime: string): Promise<void> {
	return invoke<void>("set_schedule_config", { enabled, startTime, endTime });
}

export async function updateDownloadUrl(id: string, newUrl: string): Promise<void> {
  return await invoke("update_download_url", { id, newUrl });
}

export async function saveCredential(credential: any): Promise<void> {
  return await invoke("save_credential", { credential });
}

export async function deleteCredential(domain: string): Promise<void> {
  return await invoke("delete_credential", { domain });
}

export async function getCredentials(): Promise<any[]> {
  return await invoke("get_credentials");
}

export async function getTaskLogs(id: string): Promise<string[]> {
  return await invoke("get_task_logs", { id });
}

export async function openFolder(path: string): Promise<void> {
  return await invoke("open_folder", { path });
}

export async function redownloadTask(id: string): Promise<void> {
  return await invoke("redownload_task", { id });
}
