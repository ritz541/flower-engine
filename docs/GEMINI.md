# GEMINI.md - Project Context & Instructions

## Project Overview
**The Flower Roleplay Engine** is a high-performance, split-architecture narrative system designed for immersive tabletop-style roleplaying. It separates the "Brain" (logic, RAG, and AI orchestration) from the "Face" (a blazingly fast terminal interface).

- **The Brain (Python 3.12)**: A headless FastAPI server managing SQLite state, ChromaDB vector embeddings for RAG, and LLM integrations (OpenRouter/DeepSeek).
- **The Face (Rust / Ratatui)**: A multi-threaded TUI leveraging `tokio` and `crossterm` for a responsive, 3-pane layout with typewriter animations and real-time status updates.
- **Communication**: The two components communicate via a strict WebSocket JSON protocol (V1 Schema Contract).

## Tech Stack
- **Backend**: Python 3.12+, FastAPI, Uvicorn, SQLAlchemy (SQLite), ChromaDB (Vector Search), `openai-python`, `watchdog` (folder sync).
- **Frontend**: Rust, Cargo, `ratatui` (UI), `tokio` (Async), `tokio-tungstenite` (WebSockets), `serde` (JSON).
- **Configuration**: YAML-based config for API keys, model routing, and storage paths.

## Building and Running
The project uses a unified orchestrator to launch both components simultaneously.

1.  **Environment Setup**:
    - Ensure Python 3.12+ and Rust/Cargo are installed.
    - Create a virtual environment: `python3 -m venv venv`.
    - Install dependencies: `pip install -r requirements.txt`.
    - Copy `config.yaml.example` to `config.yaml` and add your API keys (OpenRouter/DeepSeek).
2.  **Launch**:
    - Run `python run.py`.
    - This script spins up the FastAPI backend on port 8000 and then executes `cargo run` in the `tui/` directory.

## Core Commands (Inside TUI)
- `/model <name>`: Hot-swaps the active LLM.
- `/world select <id>`: Activates a world context from `worlds/*.yaml`.
- `/character select <id>`: Activates a character persona from `characters/*.yaml`.
- `/session new`: Starts a fresh narrative session with localized memory.
- `/session continue <id>`: Resumes a previous session from history.
- `/rules add <id>`: Injects a specific rulebook from `rules/*.md` into the prompt.
- `/world sync_folder <path>`: Monitors a folder for new lore files (`.txt`/`.md`) and embeds them into the world RAG in real-time.
- `/quit` or `ESC`: Gracefully shuts down both the TUI and the backend server.

## Architecture & Data Flow
1.  **Initial Load**: The Brain scans `worlds/`, `characters/`, and `rules/` for static assets.
2.  **Handshake**: Upon connection, the Brain sends a `sync_state` payload containing available models, worlds, and characters.
3.  **RAG Query**: When a user submits a prompt, the Brain performs two vector searches:
    - **World Lore**: Static or synced world data.
    - **Session Memory**: Past interactions within the current session.
4.  **Generation**: Context is injected into a system prompt defining the LLM as the "Narrator/Game Master" and the user as the "Player Character".
5.  **Streaming**: Responses are streamed via `chat_chunk` events, calculating tokens-per-second (TPS) in real-time.

## Development Conventions
- **Surgical Edits**: When modifying the Brain, ensure the WebSocket V1 Schema is preserved.
- **TUI Updates**: UI changes in `tui/src/ui.rs` should respect the `TICK_RATE` (150ms) for animations.
- **Lore Assets**: World lore should be concise; the RAG query is optimized for `n_results=2` (Lore) and `n_results=3` (Memory).
- **Error Handling**: System warnings (e.g., context bloat) should be pushed via `system_update` events to keep the user informed.
