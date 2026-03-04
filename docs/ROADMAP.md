# TODO - The Flower Roleplay Engine Enhancements

## Phase 1: Mechanical Friction (The "Game" Layer)
- [ ] **Dice Roll System**: Implement a backend function for success/failure checks (e.g., D20 logic).
- [ ] **Narrative Complications**: Force the AI to narrate specific failure states when a roll is lost via `[SYSTEM]` injection.
- [ ] **Manual Override**: Add a `/roll` command for player-initiated checks.

## Phase 2: Stateful NPCs & Factions
- [ ] **Relationship Tracking**: Add a SQLite table to store NPC/Faction standing (numerical values).
- [ ] **Persistent NPCs**: Create a system to "register" recurring NPCs so they maintain consistency across different sessions.
- [ ] **Dynamic Standing**: Automatically inject relevant NPC/Faction states into the system prompt based on the current scene.

## Phase 3: Sensory Anchoring (Prompt Refinement)
- [ ] **Object-Oriented Narration**: Update `SYSTEM RULES` to mandate the description of at least one physical object and its state in every response.
- [ ] **Environmental Clocks**: Implement backend "timers" that advance world events even when the player is inactive.
- [ ] **Atmospheric Anchors**: Add dynamic "Ambient Conditions" (weather, mood, lighting) that shift as the story progresses.

## Phase 4: Model & Technical Polish
- [ ] **Markdown Support**: Enhance the TUI to render bold, italics, and lists within chat messages.
- [ ] **Claude 3.5 Sonnet Integration**: Benchmark for superior roleplay subtext and emotional depth.
- [ ] **Log Export**: Add a command to export sessions into beautifully formatted Markdown files for reading later.
- [ ] **Soundscapes**: Explore subtle terminal-based audio cues for immersive transitions.
