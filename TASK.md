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

## Future Sprints (Backlog)

- [ ] Browser extension integration (Chrome/Firefox).
- [ ] Global and per-download speed limiters.
- [ ] Categorization rules based on file extensions.
- [ ] BitTorrent engine integration.
