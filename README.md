# Split-Architecture Roleplay Engine

A premium, highly-performant terminal-based Roleplay Engine showcasing a modern split-architecture design. 

The system is structurally divided into two distinct components that communicate securely over a strict WebSocket JSON contract:
1. **The Brain (Python 3.12)**: A headless FastAPI server managing SQLite databases, ChromaDB embeddings, generative AI API endpoints via OpenRouter, and a Watchdog file sync system.
2. **The Face (Rust / Ratatui)**: A blazingly fast, multi-threaded Terminal UI leveraging `tokio` for non-blocking asynchronous WebSockets and `crossterm` for a beautiful, responsive 3-pane layout.

## 🌟 Showcase Features

- **Multi-Turn Dynamic Context (RAG)**: The engine implements localized session memory. Interactions are evaluated, serialized, and embedded into a `session_memory` ChromaDB collection on each turn. Relevant history is natively injected into the prompt context for persistent LLM memory without Token Window bloat.
- **Lore Context Window Protections**: RAG queries are mathematically verified. If retrieved lore exceeds 1000 characters, the Brain fires a red System Warning through the WebSocket directly into the TUI, warning the user of possible AI hallucinations prior to generation streams.
- **Watchdog Deep-Sync Lore Folders**: Using the command `/world sync_folder <path>`, the Python backend attaches a structural `watchdog` to your file system. Any new `.txt` or `.md` files saved to that folder are instantaneously embedded into your world logic. 
- **Professional TUI Aesthetics**: The UI is designed utilizing a disciplined palette of Soft Cyan and Dimmed Gray padding, adaptive buffering `|` typewriter animations synced to a `500ms` internal tick rate, and a dedicated top Status Header.
- **Automated Process Orchestration**: The entire application (both the raw Python backend server and the compiled Rust UI) effortlessly powers on and securely halts through a simple, unified `run.py` wrapper.
- **The "Hacker" YAML Config**: `config.yaml` manages API routing (OpenRouter/Local vLLM), default models, and Chroma DB Paths, natively broadcasting the live supported arrays to the TUI on connection handshake.

---

## 🚀 Getting Started

### Prerequisites
- **Python 3.12+**
- **Rust / Cargo**

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/roleplay-engine.git
   cd roleplay-engine
   ```

2. **Initialize the Python Backend Environment:**
   ```bash
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   ```

3. **Configure your Engine Settings:**
   - Open `config.yaml`
   - Insert your `openai_api_key` (if using OpenRouter, this is your OpenRouter key).
   - Insert your `deepseek_api_key` to enable native routing for the official `deepseek-chat` and `deepseek-reasoner` models.

4. **Run the Orchestrator:**
   ```bash
   python run.py
   ```
   *The script will automatically detect Cargo, silently spin up the Uvicorn FastAPI listener, bind it to port 8000, and inject your terminal with the compiled Rust Ratatui Face.*

## 🕹️ TUI Commands
Inside the `Input` window, you can process local generative chat flows, or issue structural `/` commands:
- `/model <name>`: Instantly hot-swaps the generative model against the `supported_models` array in `config.yaml`.
- `/world attach_lore <text>`: Synthesizes direct input strings strictly into the `world_lore` ChromaDB embeddings.
- `/world sync_folder <path>`: Scans and physically monitors the folder directory for `.txt` assets, actively mutating the World context.
- `/quit` or `ESC`: Securely powers down the Rust interface and cascades a kill signal to the detached python server.

---

## 🧠 Architectural Contract

Communication across the bridge occurs exclusively via stringified WebSocket payloads following the V1 Schema Contract:

```json
{
  "event": "system_update | sync_state | chat_chunk | error | chat_end",
  "payload": {
    "content": "Message content or streaming text output.",
    "metadata": {
        "model": "google/gemini-2.0-pro-exp-02-05:free",
        "tokens_per_second": 45.5,
        "world_id": "world_1",
        "character_id": "char_1"
    }
  }
}
```

## Disclaimer
Note: The engine generates SQLite state tracking dynamically in `./engine.db` and `./chroma_db`. These folders are properly omitted in the `.gitignore` mapping.
