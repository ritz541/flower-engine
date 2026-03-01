import sqlite3
from pydantic import BaseModel
from typing import List, Optional
from datetime import datetime

# --- Pydantic Models ---
class World(BaseModel):
    id: str
    name: str
    lore: str

class Character(BaseModel):
    id: str
    name: str
    persona: str

class Message(BaseModel):
    id: Optional[int] = None
    role: str
    content: str
    character_id: str
    session_id: str = ""

class Session(BaseModel):
    id: str
    character_id: str
    world_id: str
    model: str
    title: str = ""          # derived from first user message
    created_at: str = ""
    last_used_at: str = ""

# --- Database Setup ---
DB_NAME = "engine.db"

def init_db():
    with sqlite3.connect(DB_NAME) as conn:
        cursor = conn.cursor()
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS worlds (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                lore TEXT NOT NULL
            )
        ''')
        # Migration: Add system_prompt if it doesn't exist
        try:
            cursor.execute("ALTER TABLE worlds ADD COLUMN system_prompt TEXT NOT NULL DEFAULT ''")
        except sqlite3.OperationalError:
            pass # Column already exists
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS characters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                persona TEXT NOT NULL
            )
        ''')
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                character_id TEXT NOT NULL,
                session_id TEXT NOT NULL DEFAULT ''
            )
        ''')
        # Migration: Add session_id if it doesn't exist
        try:
            cursor.execute("ALTER TABLE messages ADD COLUMN session_id TEXT NOT NULL DEFAULT ''")
        except sqlite3.OperationalError:
            pass # Column already exists
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL,
                world_id TEXT NOT NULL,
                model TEXT NOT NULL DEFAULT '',
                title TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL
            )
        ''')
        conn.commit()

# --- Managers ---
class WorldManager:
    def __init__(self, db_path: str = DB_NAME):
        self.db_path = db_path
    
    def add_world(self, world: World):
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                "INSERT OR REPLACE INTO worlds (id, name, lore) VALUES (?, ?, ?)",
                (world.id, world.name, world.lore)
            )
            conn.commit()
            
    def get_world(self, world_id: str) -> Optional[World]:
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute("SELECT id, name, lore FROM worlds WHERE id = ?", (world_id,))
            row = cursor.fetchone()
            if row:
                return World(id=row[0], name=row[1], lore=row[2])
            return None
            
    def get_all_worlds(self) -> List[World]:
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute("SELECT id, name, lore FROM worlds")
            return [World(id=row[0], name=row[1], lore=row[2]) for row in cursor.fetchall()]

class CharacterManager:
    def __init__(self, db_path: str = DB_NAME):
        self.db_path = db_path
        
    def add_character(self, character: Character):
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                "INSERT OR REPLACE INTO characters (id, name, persona) VALUES (?, ?, ?)",
                (character.id, character.name, character.persona)
            )
            conn.commit()
            
    def get_character(self, character_id: str) -> Optional[Character]:
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute("SELECT id, name, persona FROM characters WHERE id = ?", (character_id,))
            row = cursor.fetchone()
            if row:
                return Character(id=row[0], name=row[1], persona=row[2])
            return None

class MessageManager:
    def __init__(self, db_path: str = DB_NAME):
        self.db_path = db_path
        
    def add_message(self, message: Message):
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                "INSERT INTO messages (role, content, character_id, session_id) VALUES (?, ?, ?, ?)",
                (message.role, message.content, message.character_id, message.session_id)
            )
            conn.commit()
            
    def get_messages(self, character_id: str, session_id: str = "", limit: int = 50) -> List[Message]:
        with sqlite3.connect(self.db_path) as conn:
            if session_id:
                cursor = conn.execute(
                    "SELECT id, role, content, character_id, session_id FROM messages "
                    "WHERE session_id = ? ORDER BY id ASC LIMIT ?",
                    (session_id, limit)
                )
            else:
                cursor = conn.execute(
                    "SELECT id, role, content, character_id, '' FROM messages "
                    "WHERE character_id = ? AND session_id = '' ORDER BY id ASC LIMIT ?",
                    (character_id, limit)
                )
            return [Message(id=r[0], role=r[1], content=r[2], character_id=r[3], session_id=r[4])
                    for r in cursor.fetchall()]

    def delete_session_messages(self, session_id: str):
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("DELETE FROM messages WHERE session_id = ?", (session_id,))
            conn.commit()

class SessionManager:
    def __init__(self, db_path: str = DB_NAME):
        self.db_path = db_path

    def create_session(self, session_id: str, character_id: str, world_id: str, model: str) -> Session:
        now = datetime.utcnow().strftime("%Y-%m-%d %H:%M")
        s = Session(id=session_id, character_id=character_id, world_id=world_id,
                    model=model, title="New session", created_at=now, last_used_at=now)
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                "INSERT OR REPLACE INTO sessions (id, character_id, world_id, model, title, created_at, last_used_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?)",
                (s.id, s.character_id, s.world_id, s.model, s.title, s.created_at, s.last_used_at)
            )
            conn.commit()
        return s

    def update_title(self, session_id: str, title: str):
        short = title[:60] + ("…" if len(title) > 60 else "")
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("UPDATE sessions SET title = ? WHERE id = ?", (short, session_id))
            conn.commit()

    def touch(self, session_id: str):
        now = datetime.utcnow().strftime("%Y-%m-%d %H:%M")
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("UPDATE sessions SET last_used_at = ? WHERE id = ?", (now, session_id))
            conn.commit()

    def list_recent(self, limit: int = 10) -> List[Session]:
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                "SELECT id, character_id, world_id, model, title, created_at, last_used_at "
                "FROM sessions ORDER BY last_used_at DESC LIMIT ?", (limit,)
            )
            return [Session(id=r[0], character_id=r[1], world_id=r[2], model=r[3],
                            title=r[4], created_at=r[5], last_used_at=r[6])
                    for r in cursor.fetchall()]

    def get_session(self, session_id: str) -> Optional[Session]:
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                "SELECT id, character_id, world_id, model, title, created_at, last_used_at "
                "FROM sessions WHERE id = ?", (session_id,)
            )
            row = cursor.fetchone()
            if row:
                return Session(id=row[0], character_id=row[1], world_id=row[2], model=row[3],
                               title=row[4], created_at=row[5], last_used_at=row[6])
            return None

    def delete_session(self, session_id: str):
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("DELETE FROM sessions WHERE id = ?", (session_id,))
            conn.commit()

# Initialize db file on import
init_db()

# Global manager instances
world_manager = WorldManager()
char_manager = CharacterManager()
msg_manager = MessageManager()
session_manager = SessionManager()
