use std::time::Instant;
use crate::models::EntityInfo;
use ratatui::widgets::ListState;

#[derive(Clone, PartialEq)]
pub enum Role {
    Player,
    World,   // LLM narrator
    System,
    Error,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Clone, PartialEq)]
pub enum PopupMode {
    World,
    Character,
    Model,
    Rules,
    Session,
    Skills,
    Commands,
    None,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub current_streaming_message: String,
    pub input: String,
    pub world_id: String,
    pub character_id: String,
    pub session_id: String,
    pub status: String,
    pub is_typing: bool,
    pub scroll: u16,
    pub should_quit: bool,

    // Pro UI fields
    pub cursor_state: bool,
    pub spinner_frame: usize,
    pub tps: f64,
    pub active_model: String,
    pub active_prompt_price: f64,
    pub active_completion_price: f64,

    // Stats
    pub message_count: usize,
    pub total_tokens: u32,
    pub session_start: Instant,

    // Popup State
    pub show_popup: bool,
    pub popup_mode: PopupMode,
    pub selected_index: usize,
    pub popup_state: ListState,
    pub popup_search_query: String,
    pub available_worlds: Vec<EntityInfo>,
    pub available_characters: Vec<EntityInfo>,
    pub available_models: Vec<EntityInfo>,
    pub available_rules: Vec<EntityInfo>,
    pub available_sessions: Vec<EntityInfo>,
    pub available_commands: Vec<EntityInfo>,
    pub available_skills: Vec<EntityInfo>,
    pub active_rules: Vec<String>,
    pub active_skills: Vec<String>,
    pub command_hint: String,
}

impl App {
    pub fn new() -> App {
        let mut popup_state = ListState::default();
        popup_state.select(Some(0));
        
        App {
            messages: Vec::new(),
            current_streaming_message: String::new(),
            input: String::new(),
            world_id: "Connecting...".to_string(),
            character_id: "Connecting...".to_string(),
            session_id: String::new(),
            status: "Initializing...".to_string(),
            is_typing: false,
            scroll: 0,
            should_quit: false,

            cursor_state: true,
            spinner_frame: 0,
            tps: 0.0,
            active_model: "Unknown".to_string(),
            active_prompt_price: 0.0,
            active_completion_price: 0.0,

            message_count: 0,
            total_tokens: 0,
            session_start: Instant::now(),

            show_popup: false,
            popup_mode: PopupMode::None,
            selected_index: 0,
            popup_state,
            popup_search_query: String::new(),
            available_worlds: Vec::new(),
            available_characters: Vec::new(),
            available_models: Vec::new(),
            available_rules: Vec::new(),
            available_sessions: Vec::new(),
            available_commands: vec![
                EntityInfo { id: "/world select ".to_string(), name: "Select an active world".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/world sync_folder ".to_string(), name: "Monitor folder for lore".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/world attach_lore ".to_string(), name: "Manually add lore string".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/character select ".to_string(), name: "Select your persona".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/model ".to_string(), name: "Hot-swap the AI model".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/session new".to_string(), name: "Start a fresh session".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/session continue ".to_string(), name: "Resume a past session".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/session delete ".to_string(), name: "Delete a past session".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/rules add ".to_string(), name: "Activate a rule YAML".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/rules clear".to_string(), name: "Clear all active rules".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/skills add ".to_string(), name: "Acquire a new ability".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/skills clear".to_string(), name: "Forget all skills".to_string(), prompt_price: 0.0, completion_price: 0.0 },
                EntityInfo { id: "/quit".to_string(), name: "Exit the engine".to_string(), prompt_price: 0.0, completion_price: 0.0 },
            ],
            available_skills: Vec::new(),
            active_rules: Vec::new(),
            active_skills: Vec::new(),
            command_hint: String::new(),
        }
    }

    pub fn set_popup_index(&mut self, index: usize) {
        self.selected_index = index;
        self.popup_state.select(Some(index));
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.is_typing {
            self.input.push(c);
            self.update_command_hint();
        }
    }

    pub fn handle_backspace(&mut self) {
        if !self.is_typing {
            self.input.pop();
            self.update_command_hint();
        }
    }

    fn update_command_hint(&mut self) {
        if self.input.starts_with('/') {
            let cmds = [
                "/world select", "/world sync_folder", "/world attach_lore",
                "/character select", "/model", "/session new", 
                "/session continue", "/session delete", "/rules add", "/rules clear", "/quit"
            ];
            
            // Find first command that starts with input but isn't exact match
            self.command_hint = cmds.iter()
                .find(|&&c| c.starts_with(&self.input) && c != self.input)
                .map(|&c| c[self.input.len()..].to_string())
                .unwrap_or_default();
        } else {
            self.command_hint.clear();
        }
    }

    pub fn apply_hint(&mut self) {
        if !self.command_hint.is_empty() {
            self.input.push_str(&self.command_hint);
            self.command_hint.clear();
        }
    }

    pub fn submit_command_direct(&mut self, cmd: String) -> Option<String> {
        if cmd == "/quit" {
            self.should_quit = true;
            return None;
        }
        if cmd == "/session new" {
            // handle any local reset if needed
        }
        Some(cmd)
    }

    pub fn submit_message(&mut self) -> Option<String> {
        if self.input.is_empty() || self.is_typing {
            return None;
        }
        let msg = self.input.clone();

        if !msg.starts_with('/') {
            self.messages.push(ChatMessage {
                role: Role::Player,
                content: msg.clone(),
            });
            self.is_typing = true;
            self.message_count += 1;
            self.scroll = u16::MAX; // auto-scroll to bottom
        } else if msg.trim() == "/quit" {
            self.should_quit = true;
        }

        self.input.clear();
        Some(msg)
    }

    pub fn append_chunk(&mut self, chunk: &str) {
        self.current_streaming_message.push_str(chunk);
        // Advance spinner every chunk
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    pub fn finish_stream(&mut self) {
        if !self.current_streaming_message.is_empty() {
            self.messages.push(ChatMessage {
                role: Role::World,
                content: self.current_streaming_message.clone(),
            });
            self.message_count += 1;
            self.current_streaming_message.clear();
        }
        self.is_typing = false;
        self.cursor_state = true;
        self.scroll = u16::MAX; // auto-scroll to bottom
    }

    pub fn add_system_message(&mut self, msg: String) {
        // Detect errors vs normal system messages
        let role = if msg.starts_with('✗') || msg.to_lowercase().contains("error") {
            Role::Error
        } else {
            Role::System
        };
        self.messages.push(ChatMessage { role, content: msg });
        self.scroll = u16::MAX;
    }

    pub fn load_history(&mut self, history: Vec<(String, String)>) {
        self.messages.clear();
        for (role_str, content) in history {
            let role = match role_str.as_str() {
                "user"      => Role::Player,
                "assistant" => Role::World,
                "system"    => Role::System,
                _           => Role::System,
            };
            self.messages.push(ChatMessage { role, content });
        }
        self.message_count = self.messages.len();
        self.scroll = u16::MAX;
    }

    pub fn get_filtered_items(&self) -> Vec<EntityInfo> {
        let items = match self.popup_mode {
            PopupMode::World     => &self.available_worlds,
            PopupMode::Character => &self.available_characters,
            PopupMode::Model     => &self.available_models,
            PopupMode::Rules     => &self.available_rules,
            PopupMode::Session   => &self.available_sessions,
            PopupMode::Skills    => &self.available_skills,
            PopupMode::Commands  => &self.available_commands,
            _                    => return Vec::new(),
        };

        if self.popup_search_query.is_empty() {
            return items.clone();
        }

        let q = self.popup_search_query.to_lowercase();
        items.iter()
            .filter(|e| e.name.to_lowercase().contains(&q) || e.id.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    pub fn estimated_cost(&self) -> f64 {
        // Calculation based on per 1M tokens pricing
        (self.total_tokens as f64 / 1_000_000.0) * self.active_completion_price
    }

    pub fn session_elapsed(&self) -> String {
        let secs = self.session_start.elapsed().as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else {
            format!("{}m{}s", secs / 60, secs % 60)
        }
    }
}

pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
