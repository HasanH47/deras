# Deras Development Tasks

This document breaks down the development of the Deras Download Manager into manageable agile sprints.

## Sprint 1: Foundation & Project Setup ✅

**Goal:** Establish the core project structure, state management, and basic UI layout.

- [x] Initialize SvelteKit + Tauri project structure (already done).
- [x] Configure `shadcn-svelte` and Tailwind CSS for the UI components.
- [x] Set up basic routing and layout (Sidebar, Header, Main Content Area).
- [x] Define the Rust-Tauri IPC (Inter-Process Communication) command structure.
- [x] Create basic data models for a Download Task (URL, state, progress, size).
- [x] Set up JSON file based persistence for download history in Rust.

## Sprint 2: Core Downloading Engine (MVP Part 1) ✅

**Goal:** Implement the fundamental ability to download a file using Rust.

- [x] Implement a basic HTTP GET request using `reqwest` in Rust.
- [x] Stream download response to a file on disk (without chunks initially).
- [x] Implement Rust-to-Svelte event emission for download progress.
- [x] Build the UI component to display active downloads and progress bars.
- [x] Connect the UI "Add Download" form to the Rust backend to start a download.

## Sprint 3: Advanced Downloading (MVP Part 2) ✅

**Goal:** Implement multi-connection chunking and pause/resume functionality.

- [x] Fetch the `Accept-Ranges` and `Content-Length` headers before downloading.
- [x] Implement file splitting logic (calculating chunk ranges based on file size).
- [x] Implement concurrent chunk downloading using `tokio` and multiple `reqwest` clients.
- [x] Merge downloaded chunks into the final file upon completion.
- [x] Implement Pause/Resume logic by utilizing HTTP `Range` headers.
- [x] Update UI to support Pause/Resume/Cancel actions per download item.

## Sprint 4: Queue Management & Error Handling ✅

**Goal:** Make the download manager robust and handle multiple files gracefully.

- [x] Implement a global task queue in Rust to limit concurrent active downloads.
- [x] Add UI controls to manage the queue (move up, move down, force start).
- [x] Implement comprehensive error handling (network timeouts, disk full, etc.).
- [x] Build an automatic retry mechanism for failed transient downloads.
- [x] Design and implement error state UI and user notifications.

## Sprint 5: System Integration & Polish (Post-MVP) ✅

**Goal:** Integrate deeply with the Linux Desktop environment and refine the UX.

- [x] Implement Clipboard monitoring in Rust to detect copied URLs.
- [x] Build the "New Download Detected" popup modal triggered by clipboard.
- [x] Implement Dark/Light mode theming leveraging `shadcn-svelte`.
- [x] Add system tray icon and background running capability.
- [x] Integrate OS-level desktop notifications for completed downloads.
- [x] Add MD5/SHA-256 checksum verification tool in the UI.

## Sprint 6: Traffic Control & Scheduling

**Goal:** Give users fine-grained control over network bandwidth usage and download timing.

- [x] Implement global bandwidth rate limiting in the Rust backend.
- [x] Implement per-download overriding speed limit settings.
- [x] Build a Download Scheduler to define active download time windows (e.g., 12 AM - 6 AM).
- [x] Update UI with speed limiter inputs and schedule configuration settings.

## Sprint 7: Organization & Batching

**Goal:** Provide better tools for managing multiple downloads and visualizing performance.

- [x] Implement auto-categorization engine (Video, Audio, Document, etc.) based on file extensions.
- [x] Add Sidebar category filters in Svelte UI.
- [x] Build `BatchDownloadDialog.svelte` to parse multiple URLs from pasted text using Regex.
- [x] Build `AnalyticsDashboard.svelte` to plot total download speed over time.

## Sprint 8: Browser Integration

**Goal:** Intercept downloads seamlessly from the user's web browser without manual copying.

- [x] Build a companion cross-browser extension (Chrome, Firefox).
- [x] Implement a WebSocket server or Native Messaging host in the Rust backend.
- [x] Automatically route intercepted browser downloads to the Deras queue.

## Sprint 9: BitTorrent Support

**Goal:** Expand protocol support beyond standard HTTP/FTP to P2P networks.

- [ ] Research and integrate a Rust P2P library (e.g., `librqbit`).
- [ ] Support resolving and downloading `.torrent` files.
- [ ] Support magnet links in the "New Download" modal.
