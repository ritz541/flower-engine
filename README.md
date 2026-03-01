# 🌸 The Flower Roleplay Engine

A high-performance, split-architecture narrative system designed for immersive, modular, and hardcore tabletop-style roleplaying.

## ✨ Features

- **Split Architecture**: Decoupled Python "Brain" (FastAPI/LLM) and Rust "Face" (Ratatui TUI) communicating via WebSockets.
- **Lego Mechanic System**: Plug-and-play **Modules** to add HUDs, Stores, or Faction systems to any world.
- **Hardcore System Rules**: Built-in logic for grounded reality, NPC autonomy, and consequence-driven storytelling.
- **Dynamic Character Skills**: Acquire and track skills that physically alter the AI's narrative capabilities.
- **Live Token Streaming**: Blazingly fast, real-time response rendering with typewriter animations.
- **Session Management**: Full history persistence, session hot-swapping, and strict memory isolation.
- **Multi-Provider Support**: Seamlessly switch between OpenRouter, DeepSeek, and Groq.

## 🚀 Quick Start

### 1. Prerequisites
- Python 3.12+
- Rust & Cargo
- API Keys (OpenRouter, DeepSeek, or Groq)

### 2. Installation
```bash
git clone https://github.com/yourusername/theflower.git
cd theflower
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### 3. Setup Assets
To protect your personal lore, the `assets/` folder is gitignored. Create it by copying the example templates:
```bash
cp -r assets_example assets
```

### 4. Configuration
Copy `config.yaml.example` to `config.yaml` and add your keys:
```yaml
openai_api_key: "sk-or-..."
groq_api_key: "gsk_..."
```

### 5. Launch
```bash
chmod +x start.sh
./start.sh
```

## 🛠 Asset Structure

- `assets/worlds/`: Core setting, geography, and doomsday clocks.
- `assets/characters/`: Player personas and backgrounds.
- `assets/rules/`: Global narrative constraints (e.g., "No Magic").
- `assets/skills/`: Specific character abilities (e.g., "Hacking").
- `assets/modules/`: Modular mechanics (e.g., "System HUD").

## 📜 System Rules

The engine is governed by a permanent ruleset that ensures the world is reactive and dangerous:
1. **Grounded Reality**: No outside intervention or prophecies.
2. **Player Is Not Special**: You must earn your influence through action.
3. **Secrecy Is Absolute**: NPCs do not have access to your character sheet.
4. **Time Persists**: The world moves forward even if you wait.

## 🤝 Contributing

This project is designed to be modular. Feel free to fork and add your own **Modules** or **Themes**!

---
*Created with ❤️ for the roleplay community.*
