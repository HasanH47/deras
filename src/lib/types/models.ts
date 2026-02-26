export type DownloadState =
	| { type: "Pending" }
	| { type: "Downloading" }
	| { type: "Paused" }
	| { type: "Completed" }
	| { type: "Error"; message: string };

export interface DownloadTask {
	id: string;
	url: string;
	filename: string;
	save_path: string;
	state: DownloadState;
	downloaded_bytes: number;
	total_bytes: number;
	date_added: string;
}
