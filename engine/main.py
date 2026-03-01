import json
import asyncio
from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from engine.logger import log
from engine.database import World, Character, world_manager, char_manager
from engine.rag import rag_manager
from engine.utils import load_yaml_assets, build_ws_payload
from engine.handlers import broadcast_sync_state
from engine.commands import handle_command
from engine.llm import stream_chat_response
from engine.config import MODEL_NAME, SUPPORTED_MODELS, OPENAI_API_KEY
import engine.state as state
import httpx
import os

app = FastAPI(title="The Flower Engine")

@app.on_event("startup")
async def startup():
    # Sync Assets to Database
    for data in load_yaml_assets("assets/worlds/*.yaml"):
        w = World(id=data["id"], name=data["name"], lore=data.get("lore", ""), start_message=data.get("start_message", ""))
        world_manager.add_world(w)
        if w.lore: rag_manager.add_lore(w.id, "base_lore", w.lore)

    for data in load_yaml_assets("assets/characters/*.yaml"):
        c = Character(id=data["id"], name=data["name"], persona=data.get("persona", ""))
        char_manager.add_character(c)

    log.info(f"Engine state initialized: {len(state.available_worlds)} worlds, {len(state.available_characters)} characters, {len(state.available_modules)} modules.")

    # Fetch Models
    log.info("Fetching models...")
    try:
        async with httpx.AsyncClient() as hc:
            headers = {"Authorization": f"Bearer {OPENAI_API_KEY}"} if OPENAI_API_KEY else {}
            resp = await hc.get("https://openrouter.ai/api/v1/models", headers=headers)
            if resp.status_code == 200:
                for m in resp.json().get("data", []):
                    p = m.get("pricing", {})
                    state.AVAILABLE_MODELS.append({
                        "id": m["id"], "name": m.get("name", m["id"]),
                        "prompt_price": round(float(p.get("prompt", 0)) * 1e6, 4),
                        "completion_price": round(float(p.get("completion", 0)) * 1e6, 4)
                    })
    except Exception as e: log.error(f"Model fetch failed: {e}")
    
    # Fetch Groq Models
    log.info("Fetching Groq models...")
    from engine.config import GROQ_API_KEY, GROQ_BASE_URL
    try:
        async with httpx.AsyncClient() as hc:
            headers = {"Authorization": f"Bearer {GROQ_API_KEY}"} if GROQ_API_KEY else {}
            resp = await hc.get(f"{GROQ_BASE_URL}/models", headers=headers)
            if resp.status_code == 200:
                for m in resp.json().get("data", []):
                    state.AVAILABLE_MODELS.append({
                        "id": m["id"], "name": f"Groq: {m['id']}",
                        "prompt_price": 0.0, "completion_price": 0.0 # Groq prices vary, usually cheap/free for some tiers
                    })
    except Exception as e: log.error(f"Groq model fetch failed: {e}")

    state.AVAILABLE_MODELS.append({"id": "deepseek-chat", "name": "DeepSeek Chat", "prompt_price": 0.14, "completion_price": 0.28})
    state.AVAILABLE_MODELS.append({"id": "deepseek-reasoner", "name": "DeepSeek Reasoner", "prompt_price": 0.55, "completion_price": 2.19})

@app.websocket("/ws/rpc")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    await websocket.send_text(build_ws_payload("system_update", "✓ Engine ready.", {"status": "ok"}))
    await broadcast_sync_state(websocket)
    
    try:
        while True:
            data = await websocket.receive_text()
            try:
                msg = json.loads(data)
                prompt = msg.get("prompt", "")
            except: prompt = data
            
            if prompt.startswith("/"):
                await handle_command(prompt, websocket)
                continue

            if not state.ACTIVE_WORLD_ID or not state.ACTIVE_CHARACTER_ID or not state.ACTIVE_SESSION_ID:
                await websocket.send_text(build_ws_payload("system_update", "✗ Prepare the stage first (World, Char, Session)."))
                continue

            # RAG
            lore_list, _ = rag_manager.query_lore(state.ACTIVE_WORLD_ID, prompt, n_results=2)
            mem_key = f"{state.ACTIVE_CHARACTER_ID}_{state.ACTIVE_SESSION_ID}"
            mem_list, _ = rag_manager.query_memory(mem_key, prompt, n_results=3)
            log.info(f"RAG: Found {len(lore_list)} lore chunks and {len(mem_list)} memory chunks.")
            full_context = f"--- LORE ---\n{chr(10).join(lore_list)}\n\n--- RECENT MEMORY ---\n{chr(10).join(mem_list)}"

            # Stream
            task = asyncio.create_task(stream_chat_response(websocket, prompt, full_context, state.ACTIVE_WORLD_ID, state.ACTIVE_CHARACTER_ID, state.ACTIVE_SESSION_ID))
            
            while not task.done():
                try:
                    # Check for cancellation without blocking the entire loop indefinitely
                    raw = await asyncio.wait_for(websocket.receive_text(), timeout=0.05)
                    try:
                        cmd_msg = json.loads(raw)
                        if cmd_msg.get("prompt") == "/cancel":
                            task.cancel()
                            await websocket.send_text(build_ws_payload("system_update", "✗ Stream cancelled by user."))
                    except: pass
                except asyncio.TimeoutError:
                    continue # Task still running, no cancel message received
            
            try:
                await task
            except asyncio.CancelledError:
                log.info("Task was successfully cancelled.")
            except Exception as e:
                log.error(f"Task failed with error: {e}")

    except WebSocketDisconnect: log.info("Disconnected.")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("engine.main:app", host="0.0.0.0", port=8000, reload=True)
