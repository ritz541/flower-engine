import os
import uuid
from fastapi import WebSocket
from engine.logger import log
from engine.database import world_manager, char_manager, session_manager, msg_manager
from engine.rag import rag_manager
from engine.utils import build_ws_payload
from engine.handlers import broadcast_sync_state
import engine.state as state

async def handle_command(cmd_str: str, websocket: WebSocket):
    parts = cmd_str.split(" ", 2)
    cmd = parts[0]
    
    if cmd == "/model" and len(parts) >= 2:
        new_model = parts[1]
        if any(m["id"] == new_model for m in state.AVAILABLE_MODELS):
            state.CURRENT_MODEL = new_model
            await websocket.send_text(build_ws_payload("system_update", f"Hot-swapped model to {state.CURRENT_MODEL}", {"model": state.CURRENT_MODEL}))
        else:
            await websocket.send_text(build_ws_payload("system_update", f"Error: {new_model} is not recognized."))
    
    elif cmd == "/world" and len(parts) >= 3:
        sub = parts[1]
        if sub == "attach_lore":
            lore_text = parts[2]
            rag_manager.add_lore(state.ACTIVE_WORLD_ID, str(uuid.uuid4()), lore_text)
            await websocket.send_text(build_ws_payload("system_update", "Lore attached successfully."))
        elif sub == "select":
            state.ACTIVE_WORLD_ID = parts[2].strip()
            await websocket.send_text(build_ws_payload("system_update", f"Active world set to {state.ACTIVE_WORLD_ID}", {"world_id": state.ACTIVE_WORLD_ID}))
            await broadcast_sync_state(websocket)
            
    elif cmd == "/character" and len(parts) >= 3 and parts[1] == "select":
        state.ACTIVE_CHARACTER_ID = parts[2].strip()
        await websocket.send_text(build_ws_payload("system_update", f"Active character set to {state.ACTIVE_CHARACTER_ID}", {"character_id": state.ACTIVE_CHARACTER_ID}))
        await broadcast_sync_state(websocket)
    
    elif cmd == "/rules":
        if len(parts) >= 3 and parts[1] == "add":
            rule_id = parts[2].strip()
            if os.path.exists(f"rules/{rule_id}.yaml"):
                if rule_id not in state.ACTIVE_RULES: state.ACTIVE_RULES.append(rule_id)
                await websocket.send_text(build_ws_payload("system_update", f"✓ Rule '{rule_id}' activated", {"active_rules": state.ACTIVE_RULES}))
                await broadcast_sync_state(websocket)
        elif len(parts) >= 2 and parts[1] == "clear":
            state.ACTIVE_RULES.clear()
            await websocket.send_text(build_ws_payload("system_update", "✓ All active rules cleared", {"active_rules": state.ACTIVE_RULES}))
            await broadcast_sync_state(websocket)

    elif cmd == "/skills":
        if len(parts) >= 3 and parts[1] == "add":
            skill_id = parts[2].strip()
            if os.path.exists(f"skills/{skill_id}.yaml"):
                if skill_id not in state.ACTIVE_SKILLS: state.ACTIVE_SKILLS.append(skill_id)
                await websocket.send_text(build_ws_payload("system_update", f"✓ Skill '{skill_id}' acquired", {"active_skills": state.ACTIVE_SKILLS}))
                await broadcast_sync_state(websocket)
        elif len(parts) >= 2 and parts[1] == "clear":
            state.ACTIVE_SKILLS.clear()
            await websocket.send_text(build_ws_payload("system_update", "✓ All skills cleared", {"active_skills": state.ACTIVE_SKILLS}))
            await broadcast_sync_state(websocket)

    elif cmd == "/session":
        if len(parts) >= 2 and parts[1] == "new":
            if not state.ACTIVE_WORLD_ID or not state.ACTIVE_CHARACTER_ID:
                await websocket.send_text(build_ws_payload("system_update", "✗ Select world/char first."))
            else:
                state.ACTIVE_SESSION_ID = uuid.uuid4().hex[:12]
                session_manager.create_session(state.ACTIVE_SESSION_ID, state.ACTIVE_CHARACTER_ID, state.ACTIVE_WORLD_ID, state.CURRENT_MODEL)
                await websocket.send_text(build_ws_payload("system_update", f"✓ New session: {state.ACTIVE_SESSION_ID}"))
                await broadcast_sync_state(websocket)
        elif len(parts) >= 3 and parts[1] == "continue":
            sess_id = parts[2].strip()
            sess = session_manager.get_session(sess_id)
            if sess:
                state.ACTIVE_SESSION_ID = sess.id
                state.ACTIVE_WORLD_ID = sess.world_id
                state.ACTIVE_CHARACTER_ID = sess.character_id
                if sess.model: state.CURRENT_MODEL = sess.model
                session_manager.touch(sess_id)
                from engine.handlers import send_chat_history
                await send_chat_history(websocket, state.ACTIVE_CHARACTER_ID, state.ACTIVE_SESSION_ID)
                await broadcast_sync_state(websocket)
        elif len(parts) >= 3 and parts[1] == "delete":
            sess_id = parts[2].strip()
            session_manager.delete_session(sess_id)
            msg_manager.delete_session_messages(sess_id)
            rag_manager.delete_session_memory(sess_id)
            if state.ACTIVE_SESSION_ID == sess_id: state.ACTIVE_SESSION_ID = ""
            await websocket.send_text(build_ws_payload("system_update", f"✓ Deleted session {sess_id}"))
            await broadcast_sync_state(websocket)
