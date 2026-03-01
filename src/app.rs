use crate::models::EntityInfo;

#[derive(Clone, PartialEq)]
pub enum Role {
    Player,
    Ai,
    System,
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
    None,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub current_streaming_message: String,
    pub input: String,
    pub world_id: String,
    pub character_id: String,
    pub status: String,
    pub is_typing: bool,
    pub scroll: u16,
    pub should_quit: bool,
    
    // Pro Iteration Fields
    pub cursor_state: bool,
    pub tps: f64,
    pub active_model: String,
    
    // Popup State
    pub show_popup: bool,
    pub popup_mode: PopupMode,
    pub selected_index: usize,
    pub popup_search_query: String,
    pub available_worlds: Vec<EntityInfo>,
    pub available_characters: Vec<EntityInfo>,
    pub available_models: Vec<EntityInfo>,
}

impl App {
    pub fn new() -> App {
        App {
            messages: Vec::new(),
            current_streaming_message: String::new(),
            input: String::new(),
            world_id: "Connecting...".to_string(),
            character_id: "Connecting...".to_string(),
            status: "Initializing...".to_string(),
            is_typing: false,
            scroll: 0,
            should_quit: false,
            
            cursor_state: true,
            tps: 0.0,
            active_model: "Unknown".to_string(),
            
            show_popup: false,
            popup_mode: PopupMode::None,
            selected_index: 0,
            popup_search_query: String::new(),
            available_worlds: Vec::new(),
            available_characters: Vec::new(),
            available_models: Vec::new(),
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.is_typing {
            self.input.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        if !self.is_typing {
            self.input.pop();
        }
    }

    pub fn submit_message(&mut self) -> Option<String> {
        if self.input.is_empty() || self.is_typing {
            return None;
        }
        let msg = self.input.clone();
        
        // Command check prevents showing local `/` commands as chat
        if !msg.starts_with('/') {
            self.messages.push(ChatMessage {
                role: Role::Player,
                content: msg.clone(),
            });
            self.is_typing = true; // Block input while waiting for response
            
            // Auto-scroll to bottom on new message
            self.scroll = self.messages.len().saturating_mul(2) as u16;
        }
        
        self.input.clear();
        Some(msg)
    }

    pub fn append_chunk(&mut self, chunk: &str) {
        self.current_streaming_message.push_str(chunk);
    }

    pub fn finish_stream(&mut self) {
        if !self.current_streaming_message.is_empty() {
            self.messages.push(ChatMessage {
                role: Role::Ai,
                content: self.current_streaming_message.clone(),
            });
            self.current_streaming_message.clear();
        }
        self.is_typing = false;
        self.cursor_state = true; // Reset cursor to solid for next interaction
    }
    
    pub fn add_system_message(&mut self, msg: String) {
        self.messages.push(ChatMessage {
            role: Role::System,
            content: msg,
        });
        
        // Auto-scroll
        self.scroll = self.messages.len().saturating_mul(2) as u16;
    }

    pub fn get_filtered_items(&self) -> Vec<EntityInfo> {
        let items = match self.popup_mode {
            PopupMode::World => &self.available_worlds,
            PopupMode::Character => &self.available_characters,
            PopupMode::Model => &self.available_models,
            _ => return Vec::new(),
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
}
