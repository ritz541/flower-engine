import time
import json
import asyncio
import uuid
import yaml
from fastapi import WebSocket
from openai import AsyncOpenAI
from engine.logger import log
from engine.config import OPENAI_BASE_URL, OPENAI_API_KEY, DEEPSEEK_API_KEY
from engine.database import char_manager, world_manager, msg_manager, session_manager, Message
from engine.rag import rag_manager
from engine.prompt import build_system_prompt
from engine.utils import build_ws_payload
import engine.state as state

# OpenAI Client — used for OpenRouter (compatible API)
client = AsyncOpenAI(
    base_url=OPENAI_BASE_URL,
    api_key=OPENAI_API_KEY,
    default_headers={
        "HTTP-Referer": "https://github.com/theflower",
        "X-Title": "The Flower Roleplay Engine",
    }
)

# Deepseek Client (Official API)
ds_client = AsyncOpenAI(
    base_url="https://api.deepseek.com",
    api_key=DEEPSEEK_API_KEY
)

async def stream_chat_response(ws: WebSocket, prompt: str, context: str, world_id: str, char_id: str, session_id: str = ""):
    character = char_manager.get_character(char_id)
    char_name    = character.name    if character else "a wanderer"
    char_persona = character.persona if character else "A mysterious figure."

    # Build injected rules block
    rules_block = ""
    if state.ACTIVE_RULES:
        loaded_texts = []
        for rule_id in state.ACTIVE_RULES:
            try:
                with open(f"rules/{rule_id}.yaml", "r", encoding="utf-8") as f:
                    data = yaml.safe_load(f)
                    if data and "prompt" in data:
                        loaded_texts.append(data["prompt"].strip())
            except Exception: pass
        if loaded_texts:
            rules_block = "### UNIVERSAL LAWS ###\n" + "\n\n".join(loaded_texts)

    # Build injected skills block
    skills_block = ""
    if state.ACTIVE_SKILLS:
        loaded_skills = []
        for skill_id in state.ACTIVE_SKILLS:
            try:
                with open(f"skills/{skill_id}.yaml", "r", encoding="utf-8") as f:
                    data = yaml.safe_load(f)
                    if data and "prompt" in data:
                        skill_name = data.get("name", skill_id)
                        loaded_skills.append(f"- {skill_name}: {data['prompt'].strip()}")
            except Exception: pass
        if loaded_skills:
            skills_block = "### CHARACTER ABILITIES ###\n" + "\n".join(loaded_skills)

    system_prompt = build_system_prompt(char_name, char_persona, rules_block, skills_block, context)
    
    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": prompt}
    ]
    
    log.info(f"Initiating stream for char {char_id} in world {world_id}")
    full_content = ""
    total_tokens = 0

    try:
        active_client = ds_client if state.CURRENT_MODEL.startswith("deepseek-") else client
        response = await active_client.chat.completions.create(
            model=state.CURRENT_MODEL,
            messages=messages,
            stream=True
        )

        start_time = None
        async for chunk in response:
            if not start_time: start_time = time.time()
            delta = chunk.choices[0].delta.content
            if delta:
                full_content += delta
                total_tokens += 1
                elapsed = time.time() - start_time
                tps = total_tokens / elapsed if elapsed > 0 else 0.0
                metadata = {"model": state.CURRENT_MODEL, "tokens_per_second": round(tps, 2), "world_id": world_id}
                await ws.send_text(build_ws_payload("chat_chunk", delta, metadata))

    except asyncio.CancelledError:
        log.info(f"Stream cancelled after {total_tokens} tokens.")
    except Exception as e:
        log.error(f"Error during streaming: {e}")
        await ws.send_text(build_ws_payload("error", str(e)))
        return
    finally:
        await ws.send_text(build_ws_payload("chat_end", "", {"total_tokens": total_tokens}))

    if full_content:
        msg_manager.add_message(Message(role="user", content=prompt, character_id=char_id, session_id=session_id))
        msg_manager.add_message(Message(role="assistant", content=full_content, character_id=char_id, session_id=session_id))
        
        memory_key  = f"{char_id}_{session_id}" if session_id else char_id
        rag_manager.add_memory(memory_key, str(uuid.uuid4()), f"User: {prompt}\nAI: {full_content}")

        if session_id:
            session_manager.touch(session_id)
            sess = session_manager.get_session(session_id)
            if sess and sess.title in ("", "New session"):
                session_manager.update_title(session_id, prompt)
