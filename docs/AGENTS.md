# AGENTS.md - The Flower Roleplay Engine

This file provides guidance for agentic coding agents working on this codebase.

## Project Overview

The Flower is a split-architecture narrative system with:
- **Python Brain** (`engine/`): FastAPI backend with WebSocket server, SQLite database, LLM integration
- **Rust Face** (`tui/`): Ratatui-based terminal UI communicating via WebSockets

## Build Commands

### Python Backend (engine/)

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Run the backend server (with auto-reload)
python -m uvicorn engine.main:app --host 0.0.0.0 --port 8000 --reload

# Run a single test (WebSocket client test)
python engine/test_client.py
```

### Rust TUI (tui/)

```bash
cd tui

# Build the TUI
cargo build

# Run the TUI
cargo run

# Run with release optimizations
cargo run --release
```

### Full System

```bash
# Start both backend and TUI
chmod +x start.sh
./start.sh
```

### Running a Single Test

There is no formal pytest setup. The only test is `engine/test_client.py` - a WebSocket client that:
1. Connects to `ws://localhost:8000/ws/rpc`
2. Expects handshake and sync_state messages
3. Sends a test prompt and verifies streaming response

To run: Start the backend first, then `python engine/test_client.py`

## Code Style Guidelines

### Python (`engine/`)

**Imports:**
- Standard library first, then third-party, then local
- Use explicit relative imports: `from engine.logger import log`
- Group imports logically within files

**Formatting:**
- 4 spaces indentation
- Maximum line length: 120 characters
- Use f-strings for string formatting
- Use trailing commas in multi-line structures

**Types:**
- Use Pydantic `BaseModel` for all data models
- Use type hints on all function signatures
- Use `Optional[T]` for nullable types
- Prefer explicit types over `Any`

**Naming Conventions:**
- Modules: `snake_case.py`
- Classes: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `UPPER_SNAKE_CASE`
- Private members: prefix with underscore

**Error Handling:**
- Use try/except with specific exception types
- Log errors via `from engine.logger import log`
- Use bare `except:` only when necessary (never in new code)
- Propagate errors appropriately or return sensible defaults

**Async Patterns:**
- Use `async def` for all WebSocket handlers
- Use `asyncio` for concurrent operations
- Use `asyncio.create_task()` for fire-and-forget background tasks
- Handle `asyncio.CancelledError` explicitly

**Database:**
- Use SQLite with the pattern in `database.py`
- Use Pydantic models for all table rows
- Use context managers for connections: `with sqlite3.connect(...) as conn`
- Include migration logic for schema changes

### Rust (`tui/`)

**Dependencies:**
- `crossterm` - Terminal input/output
- `ratatui` - TUI widget library  
- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket client
- `serde` with derive - Serialization
- `futures-util` - Async utilities

**Naming:**
- Types: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `UPPER_SNAKE_CASE`
- Enums: `PascalCase` for variants

**Error Handling:**
- Use `Result<T, Box<dyn Error>>` for main functions
- Use `?` operator for error propagation
- Use `unwrap()` only in tests or when failure is impossible

**Async:**
- Use `#[tokio::main]` for async main
- Use `tokio::select!` for concurrent event processing
- Use channels (`mpsc::unbounded_channel`) for message passing

**WebSocket Protocol:**
- Messages are JSON with format: `{"event": "...", "payload": {"content": "...", "metadata": {...}}}`
- Known events: `sync_state`, `chat_history`, `system_update`, `chat_chunk`, `chat_end`, `error`
- See `tui/src/models.rs` for message schemas

### WebSocket Communication

The Python backend and Rust TUI communicate via JSON messages:

```python
# Python: Send message
json.dumps({
    "event": "event_name",
    "payload": {
        "content": "message text",
        "metadata": {"key": "value"}
    }
})
```

### Asset Files

- Location: `assets/`
- Format: YAML (`.yaml`)
- Structure: Each asset has an `id` field plus type-specific fields
- Load via: `engine/utils.py::load_yaml_assets(pattern)`

## Project Structure

```
theflower/
├── engine/                 # Python FastAPI backend
│   ├── main.py            # FastAPI app + WebSocket handler
│   ├── commands.py        # Command handlers (/model, /world, etc.)
│   ├── database.py        # SQLite + Pydantic models
│   ├── config.py          # Configuration loader
│   ├── state.py           # Global state + persistence
│   ├── rag.py             # RAG/chromadb integration
│   ├── llm.py             # LLM streaming logic
│   ├── handlers.py        # Broadcast helpers
│   ├── utils.py           # Utilities
│   └── logger.py          # Logging setup
├── tui/                    # Rust TUI
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        # Entry point + event loop
│       ├── app.rs         # App state + logic
│       ├── models.rs      # WebSocket message types
│       ├── ws.rs          # WebSocket client
│       └── ui/            # UI rendering
├── assets/                # Game assets (gitignored)
│   ├── worlds/
│   ├── characters/
│   ├── rules/
│   ├── skills/
│   └── modules/
├── config.yaml            # Configuration
├── engine.db              # SQLite database
└── persist.json           # State persistence
```

## Common Development Tasks

### Adding a New Command
1. Add handler in `engine/commands.py`
2. Parse command string with `parts = cmd_str.split(" ", 2)`
3. Send response via `await websocket.send_text(build_ws_payload(...))`

### Adding a New WebSocket Event
1. Define event in both Python and Rust
2. Python: Send via `build_ws_payload()`
3. Rust: Handle in `main.rs::run_app()` match statement

### Modifying Database Schema
1. Add migration logic in `database.py::init_db()`
2. Use `try/except sqlite3.OperationalError` pattern for ALTER TABLE
3. Update Pydantic models accordingly

## Configuration

Edit `config.yaml` for:
- `database_path`: SQLite storage location
- `default_model`: Default LLM model
- `supported_models`: List of available models
- API keys for providers (OpenRouter, DeepSeek, Groq)

Environment variables can override config: `MODEL_NAME`, `OPENAI_API_KEY`, etc.
