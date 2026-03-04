# 🌸 The Flower Roleplay Engine

A high-performance, split-architecture narrative system designed for immersive, modular, and hardcore tabletop-style roleplaying. 

> **✨ Fully Vibecoded Application** — *Built at the speed of thought with AI-first engineering.*

---

## ✨ Features

- **Split Architecture**: Decoupled Python "Brain" (FastAPI/LLM) and Rust "Face" (Ratatui TUI) communicating via WebSockets.
- **Hardcore System Rules**: Built-in logic for grounded reality, NPC autonomy, and consequence-driven storytelling.
- **Live Token Streaming**: Blazingly fast, real-time response rendering with typewriter animations.
- **Session Management**: Full history persistence, session hot-swapping, and strict memory isolation.

## 🏗 Architecture

```text
    [ THE FACE ]             [ THE BRAIN ]
    (Rust / Ratatui)         (Python / FastAPI)
          |                         |
    TUI Interface <--- WebSocket ---> LLM Orchestrator
          |            (JSON V1)            |
    Async Input                     RAG (ChromaDB)
    Event Loop                      SQLite Persistence
```

## 🛠 Technical Stack

- **The Brain (Backend)**: 
  - **Python 3.12+** / **FastAPI** (Asynchronous orchestration)
  - **SQLite** (Session & character persistence)
  - **ChromaDB** (Vector storage for RAG)
  - **SentenceTransformers (`all-MiniLM-L6-v2`)** (Local embeddings)
- **The Face (Frontend)**:
  - **Rust** / **Ratatui** (Blazingly fast terminal UI)
  - **Tokio** (Multi-threaded async events)
- **Communication**: WebSocket JSON Protocol (V1)

## 💻 System Requirements

- **OS**: Linux, macOS, or Windows (via WSL2 recommended)
- **Memory**: 4GB+ RAM (Embeddings run on CPU for maximum compatibility)
- **Disk**: ~1GB (Setup is optimized to avoid heavy CUDA libraries)
- **Software**: 
  - Python 3.12 or newer
  - Rust & Cargo (Latest stable)
  - `git` for cloning
- **API Keys**: At least one from OpenRouter, Google Gemini, Groq, or DeepSeek.

## 🚀 Quick Start

### 1. Prerequisites
- **Python 3.12+**
- **Rust & Cargo** (for the TUI)
- **API Keys** (OpenRouter, DeepSeek, or Groq)

### 2. Automatic Setup
We've provided a setup script to handle the virtual environment and initial asset creation:

```bash
git clone https://github.com/ritz541/flower-engine.git
cd flower-engine
chmod +x setup.sh
./setup.sh
```

### 3. Configuration
1. Open `config.yaml` in your preferred editor.
2. Add your API keys for the providers you wish to use (OpenRouter is recommended for the widest model support).

### 4. Launch
```bash
./start.sh
```

## 🛠 Asset Structure

- `assets/worlds/`: Core setting, geography, and doomsday clocks.
- `assets/characters/`: Player personas and backgrounds.
- `assets/rules/`: Global narrative constraints (e.g., "No Magic").

## 📜 System Rules

The engine is governed by a permanent ruleset that ensures the world is reactive and dangerous:
1. **Grounded Reality**: No outside intervention or prophecies.
2. **Player Is Not Special**: You must earn your influence through action.
3. **Secrecy Is Absolute**: NPCs do not have access to your character sheet.
4. **Time Persists**: The world moves forward even if you wait.

## 🗺 Roadmap

The project is evolving! See [docs/ROADMAP.md](docs/ROADMAP.md) for upcoming features like:
- Dice Roll System (D20 Logic)
- Stateful NPCs & Relationship Tracking
- Atmospheric Environmental Anchors
- Markdown Rendering in TUI

## 🤝 Contributing

This project is built to be extensible. Feel free to fork and add your own **Worlds**, **Characters**, or **Themes**! See [docs/](docs/) for more internal documentation.

## ⚖ License

Distributed under the MIT License. See `LICENSE` for more information.

---
*Created with ❤️ and **fully vibecoded** for the roleplay community.*
