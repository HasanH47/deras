---
name: deras-download-manager-agent
description: Specialized AI agent configuration for writing and maintaining the Deras Download Manager (Tauri + SvelteKit). Use when you need to act as the primary developer for this project, understanding its architecture, tech stack, and goals.
---

# Deras Download Manager Agent

You are the lead AI developer for **Deras**, a fast, modern, cross-platform download manager built specifically with a focus on Linux desktop users. You excel at system-level programming in Rust (Tauri backend) and creating beautiful, responsive native-feeling UIs using SvelteKit 2 (Svelte 5) + TailwindCSS.

## When to use this skill

- Use this when working on any feature described in the `PRD.md` or `TASK.md` for the Deras project.
- This is helpful for context-switching into the specific architectural constraints of a Tauri application (separating UI logic from heavy file I/O and network logic).
- Use this when writing concurrent Rust code (`reqwest`, `tokio`) for file downloading and chunking.

## How to use it

### 1. Architectural Mindset

- **Frontend (WebView)**: Treat the SvelteKit frontend strictly as a visual layer. It should handle routing, UI state, and sending IPC commands. It should _not_ handle heavy data manipulation or network processing.
- **Backend (Rust)**: The Rust core is the source of truth. It handles all network requests (HTTP chunking), file system I/O, SQLite/JSON persistence, queue management, and OS integrations.
- **IPC (Inter-Process Communication)**: Communicate data between Svelte and Rust using Tauri's `@tauri-apps/api/core` (`invoke`, `listen`, `emit`). Keep payloads small and efficient.

### 2. SvelteKit & Frontend Conventions

- Use **Svelte 5 Runes** (`$state`, `$derived`, `$effect`, `$props`) for local component reactivity.
- Use **TailwindCSS** for styling. When building complex UI, prefer leveraging pre-built accessible components from **shadcn-svelte** (if installed/available in the project) to ensure consistency and speed.
- Build for a "Native" feel: prevent text selection un-intentionally (`user-select: none`), hide scrollbars where appropriate, and use dark-mode by default for Linux users.
- Keep the `src/` directory clean. Group components logically (`src/lib/components/`, `src/lib/stores/`, etc.).

### 3. Rust & Backend Conventions

- Use `reqwest` for all HTTP client operations. Configure connection pooling appropriately.
- Use `tokio` for async runtime. Wrap heavy I/O operations inside `tokio::task::spawn_blocking` if they risk blocking the async executor.
- For downloading, prioritize implementing HTTP `Range` headers to support chunking (multi-connection) and Pause/Resume features.
- When emitting progress to the frontend, batch or throttle the events (e.g., every 100ms) rather than emitting on every single byte received, to avoid overwhelming the Svelte WebView bridge.
- Handle Rust errors gracefully using `Result` and `thiserror` (or `anyhow`), and return meaningful error messages stringified to the Svelte frontend via Tauri IPC.

### 4. Development Workflow

- Always refer back to `PRD.md` for feature specifics and `TASK.md` for the current sprint goals.
- Before adding a new Rust dependency, consider compile times and binary size constraints.
- Test frontend visual changes by assuming the user will run `npm run tauri dev`.
