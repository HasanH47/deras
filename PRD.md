# Product Requirements Document (PRD)

## Project Name: Deras

**Platform**: Linux Desktop (Cross-platform capability via Tauri)
**Category**: Download Manager

## 1. Introduction

Deras is a modern, fast, and feature-rich download manager tailored primarily for Linux desktop users, but built with cross-platform capabilities in mind. Built with Tauri and SvelteKit, it aims to provide a lightweight, native-feeling experience while maximizing download speeds and reliability through concurrent network programming.

## 2. Tech Stack

- **Frontend**: SvelteKit 2 (Svelte 5) + Vite + TailwindCSS / shadcn-svelte (recommended for styling)
- **Backend / Core**: Rust (Tauri 2)
- **Networking**: `reqwest` (Rust) for HTTP client, handling multi-connection downloads.
- **State Management**: Svelte runes (`$state`, `$derived`) / Svelte stores.

## 3. Core Features - Minimum Viable Product (MVP)

The MVP focuses on getting the core downloading engine rock-solid.

1. **Multi-Connection / Chunk Downloading**
   - Split large files into multiple chunks and download them concurrently to maximize bandwidth utilization.
   - Dynamic chunk allocation based on file size and server support.
2. **Pause & Resume Capability**
   - Support for HTTP `Range` headers to pause ongoing downloads and resume them without restarting.
   - Persistent download state (saving progress to disk) so downloads can survive app restarts.
3. **Queue & Concurrency Management**
   - A task queue system that limits the number of active concurrent downloads.
   - Prioritization of downloads (move up/down the queue).
4. **Basic Error Handling & Auto-Retry**
   - Automatic retries on transient network errors or server timeouts.
   - Clear error states in the UI when a download fails permanently.

## 4. Advanced Features (Post-MVP)

Enhancements to make Deras a daily driver and competitive with established download managers.

1. **Browser Integration (Extension)**
   - Extensions for Chrome, Firefox, and Chromium-based browsers to intercept download links automatically.
   - Communicate securely with the desktop app via WebSockets or Native Messaging.
2. **Speed Limiter**
   - Global bandwidth capping and per-download speed limits to prevent network monopolization.
3. **Checksum Verification**
   - Automatic SHA-256 / MD5 validation post-download to ensure file integrity.
   - UI input for users to provide expected checksums before the download finishes.
4. **Clipboard Monitor**
   - Background monitoring of the system clipboard.
   - Auto-detect actionable URLs (e.g., `.mp4`, `.zip`, `.iso`, `.pdf`) and trigger a "New Download" popup modal.

## 5. Potential Improvements & Future Roadmap

Suggestions for making Deras even better:

- **Categorization & Rules**: Auto-sort downloaded files into folders based on file extensions (e.g., Documents, Videos, Compressed).
- **Download Scheduler**: Allow scheduling downloads for off-peak hours (e.g., midnight to 6 AM).
- **Site Exporter / Batch Download**: Ability to scrape a webpage for media links and download them in a batch.
- **BitTorrent Support**: Expand beyond HTTP/FTP by integrating a Rust torrent engine to handle magnet links and `.torrent` files.
- **Detailed Analytics & Graphs**: Visual representations of download speeds over time, active connections, and total bandwidth usage.
- **Dark Mode & Theming**: Robust aesthetic customization since it's targeted at Linux users who love cohesive window theming.
- **System Tray Integration**: Run in the background silently, sending system notifications upon download completion.

## 6. Project Architecture (Concept)

- **Frontend (Tauri WebView)**: Acts strictly as a UI layer. Sends IPC commands (e.g., `start_download`, `pause_download`) to the Rust backend and listens to progress events.
- **Backend (Rust Core)**: Handles file I/O, `reqwest` connection pooling, SQLite/JSON state persistence, and OS-level integrations (clipboard, tray, notifications).
- **Inter-Process Communication (IPC)**: The bridge between the Svelte frontend and the Rust engine via Tauri's native message passing.
