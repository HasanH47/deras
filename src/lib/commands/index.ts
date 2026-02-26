import { invoke } from "@tauri-apps/api/core";
import type { DownloadTask } from "$lib/types/models";

export async function getDownloads(): Promise<DownloadTask[]> {
	return invoke<DownloadTask[]>("get_downloads");
}

export async function addDownload(url: string, save_path: string): Promise<DownloadTask> {
	return invoke<DownloadTask>("add_download", { url, savePath: save_path });
}

export async function removeDownload(id: string): Promise<void> {
	return invoke<void>("remove_download", { id });
}
