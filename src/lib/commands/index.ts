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
