import json
from fastapi import WebSocket
from engine.logger import log
from engine.database import msg_manager, session_manager
from engine.utils import build_ws_payload
import engine.state as state


async def broadcast_sync_state(ws: WebSocket):
    recent_sessions = [
        {"id": s.id, "name": f"[{s.world_id}] {s.character_id}: {s.title[:30]}"}
        for s in session_manager.list_recent(15)
    ]
    await ws.send_text(
        build_ws_payload(
            "sync_state",
            "State synchronized",
            {
                "model": state.CURRENT_MODEL,
                "model_confirmed": state.MODEL_CONFIRMED,
                "world_id": state.ACTIVE_WORLD_ID,
                "character_id": state.ACTIVE_CHARACTER_ID,
                "session_id": state.ACTIVE_SESSION_ID,
                "available_worlds": state.available_worlds,
                "available_characters": state.available_characters,
                "available_models": state.AVAILABLE_MODELS,
                "available_rules": state.available_rules,
                "active_rules": state.ACTIVE_RULES,
                "available_sessions": recent_sessions,
            },
        )
    )


async def send_chat_history(ws: WebSocket, char_id: str, session_id: str):
    messages = msg_manager.get_messages(char_id, session_id)
    history = [{"role": m.role, "content": m.content} for m in messages]
    await ws.send_text(
        json.dumps(
            {
                "event": "chat_history",
                "payload": {"content": "", "metadata": {"history": history}},
            }
        )
    )
