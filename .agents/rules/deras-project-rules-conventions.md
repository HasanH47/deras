---
trigger: always_on
---

# Deras Project Rules & Conventions

This document outlines the coding standards, architectural decisions, and conventions for the **Deras** Download Manager project. All contributors (including AI agents) must adhere to these rules to maintain code quality and consistency.

## 1. Tech Stack Overview

- **Frontend**: SvelteKit 2 using Svelte 5 (Runes), TailwindCSS, TypeScript.
- **Backend/Core**: Rust, Tauri 2, `reqwest`, `tokio`.
- **Package Manager**: bun.

## 2. Frontend Conventions (Svelte 5 + UI)

### 2.1. State Management (Svelte 5 Runes)

- **Always** use Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`) instead of old Svelte 3/4 reactivity (`let`, `$:`, `export let`).
- Example component structure:

  ```svelte
  <script lang="ts">
    let { title, description } = $props<{ title: string; description: string }>();
    let count = $state(0);
    let double = $derived(count * 2);

    $effect(() => {
      console.log(`Count changed to ${count}`);
    });
  </script>
  ```

### 2.2. Styling & UI Components

- Use **TailwindCSS** for all styling. Avoid writing custom CSS in `<style>` blocks unless absolutely necessary for complex animations or native-feel tweaks.
- Prefer **shadcn-svelte** components for UI elements (buttons, dialogs, progress bars, etc.) to ensure a consistent, accessible, and high-quality user experience.
- **Desktop Target**: The app is for desktop (Linux primarily).
  - Disable text selection globally where inappropriate: `user-select: none`.
  - Use custom scrollbars or hide them entirely if it fits the design.
  - Make sure drag-and-drop areas are clearly defined.

### 2.3. Project Structure

- `src/lib/components/`: Reusable UI components.
- `src/lib/types/`: TypeScript interfaces and definitions (especially for IPC payloads).
- `src/lib/commands/`: Wrappers around Tauri IPC calls using `@tauri-apps/api/core` (`invoke`).

## 3. Backend Conventions (Rust + Tauri)

### 3.1. Async & Networking

- Use `tokio` as the async runtime.
- For heavy I/O operations (like writing chunks to disk), wrap them in `tokio::task::spawn_blocking` to avoid blocking the async executor.
- Use `reqwest` for HTTP requests. Manage connections efficiently using a shared `reqwest::Client` (connection pool).

### 3.2. Error Handling

- Never `unwrap()` or `expect()` in production code unless you can absolutely guarantee it won't panic.
- Use the `thiserror` crate to define custom error types (e.g., `DownloadError`) and `Result` for returning errors.
- Any error returned from a Tauri command to the Svelte frontend **must be serializable**.

  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum AppError {
      #[error("Network error: {0}")]
      Network(#[from] reqwest::Error),
      // ...
  }

  // Convert custom error to String for frontend IPC
  impl serde::Serialize for AppError {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
          S: serde::ser::Serializer,
      {
          serializer.serialize_str(self.to_string().as_ref())
      }
  }
  ```

### 3.3. IPC (Inter-Process Communication) payload

- Keep payloads sent between Rust and Webview as small as possible.
- Throttle high-frequency events (like download progress updates) to 10-100ms intervals instead of sending an event for every chunk downloaded.

## 4. Git & Commit Workflow

- Follow **Conventional Commits** for commit messages:
  - `feat: add pause/resume support`
  - `fix: correct percentage calculation`
  - `refactor: extract reqwest client to state`
  - `chore: update dependencies`
- Keep Pull Requests/Branches focused on a single feature or fix from `TASK.md`.

## 5. File System & Persistent Data

- Store download history and configurations in the standard OS AppData directory (accessible via Tauri's `path` API) rather than hardcoding paths.
- For the MVP, `serde_json` or `sqlite` (via `rusqlite` or `sqlx`) can be used to persist the state of paused/active downloads so they survive application restarts.
