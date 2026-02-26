export type DownloadState =
	| { type: "Pending" }
	| { type: "Downloading" }
	| { type: "Paused" }
	| { type: "Scheduled" }
	| { type: "Completed" }
	| { type: "Error"; message: string };

export type FilterMode =
  | "all"
  | "downloading"
  | "completed"
  | "Video"
  | "Audio"
  | "Document"
  | "Archive"
  | "Application"
  | "Image"
  | "Other"
  | "analytics";

export interface ChunkState {
	id: number;
	start_byte: number;
	end_byte: number;
	downloaded: number;
	is_complete: boolean;
}

export interface DownloadTask {
	id: string;
	url: string;
	filename: string;
	save_path: string;
	state: DownloadState;
	downloaded_bytes: number;
	total_bytes: number;
	category: "Video" | "Audio" | "Document" | "Archive" | "Application" | "Image" | "Other";
	date_added: string;
	supports_range: boolean;
	chunks: ChunkState[] | null;
	speed_limit_bytes?: number | null;
	is_torrent: boolean;
	info_hash: string | null;
}
