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

## Sprint 2: Core Downloading Engine (MVP Part 1)

**Goal:** Implement the fundamental ability to download a file using Rust.

- [ ] Implement a basic HTTP GET request using `reqwest` in Rust.
- [ ] Stream download response to a file on disk (without chunks initially).
- [ ] Implement Rust-to-Svelte event emission for download progress.
- [ ] Build the UI component to display active downloads and progress bars.
- [ ] Connect the UI "Add Download" form to the Rust backend to start a download.

## Sprint 3: Advanced Downloading (MVP Part 2)

**Goal:** Implement multi-connection chunking and pause/resume functionality.

- [ ] Fetch the `Accept-Ranges` and `Content-Length` headers before downloading.
- [ ] Implement file splitting logic (calculating chunk ranges based on file size).
- [ ] Implement concurrent chunk downloading using `tokio` and multiple `reqwest` clients.
- [ ] Merge downloaded chunks into the final file upon completion.
- [ ] Implement Pause/Resume logic by utilizing HTTP `Range` headers.
- [ ] Update UI to support Pause/Resume/Cancel actions per download item.

## Sprint 4: Queue Management & Error Handling

**Goal:** Make the download manager robust and handle multiple files gracefully.

- [ ] Implement a global task queue in Rust to limit concurrent active downloads.
- [ ] Add UI controls to manage the queue (move up, move down, force start).
- [ ] Implement comprehensive error handling (network timeouts, disk full, etc.).
- [ ] Build an automatic retry mechanism for failed transient downloads.
- [ ] Design and implement error state UI and user notifications.

## Sprint 5: System Integration & Polish (Post-MVP)

**Goal:** Integrate deeply with the Linux Desktop environment and refine the UX.

- [ ] Implement Clipboard monitoring in Rust to detect copied URLs.
- [ ] Build the "New Download Detected" popup modal triggered by clipboard.
- [ ] Implement Dark/Light mode theming leveraging `shadcn-svelte`.
- [ ] Add system tray icon and background running capability.
- [ ] Integrate OS-level desktop notifications for completed downloads.
- [ ] (Optional) Add MD5/SHA-256 checksum verification tool in the UI.

## Future Sprints (Backlog)

- [ ] Browser extension integration (Chrome/Firefox).
- [ ] Global and per-download speed limiters.
- [ ] Categorization rules based on file extensions.
- [ ] BitTorrent engine integration.
