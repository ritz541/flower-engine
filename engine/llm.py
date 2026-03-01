import time
import json
import asyncio
import uuid
import yaml
from fastapi import WebSocket
from openai import AsyncOpenAI
from engine.logger import log
from engine.config import OPENAI_BASE_URL, OPENAI_API_KEY, DEEPSEEK_API_KEY, GROQ_API_KEY, GROQ_BASE_URL
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

# Groq Client
groq_client = AsyncOpenAI(
    base_url=GROQ_BASE_URL,
    api_key=GROQ_API_KEY
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
                with open(f"assets/rules/{rule_id}.yaml", "r", encoding="utf-8") as f:
                    data = yaml.safe_load(f)
                    if data and "prompt" in data:
                        loaded_texts.append(data["prompt"].strip())
            except Exception: pass
        if loaded_texts:
            rules_block = "### UNIVERSAL LAWS ###\n" + "\n\n".join(loaded_texts)

    # Build injected modules block
    modules_block = ""
    if state.ACTIVE_MODULES:
        loaded_mods = []
        for mod_id in state.ACTIVE_MODULES:
            try:
                with open(f"assets/modules/{mod_id}.yaml", "r", encoding="utf-8") as f:
                    data = yaml.safe_load(f)
                    if data and "prompt" in data:
                        mod_name = data.get("name", mod_id)
                        loaded_mods.append(f"- {mod_name}: {data['prompt'].strip()}")
            except Exception: pass
        if loaded_mods:
            modules_block = "### WORLD MODULES ###\n" + "\n".join(loaded_mods)

    # Build injected skills block
    skills_block = ""
    if state.ACTIVE_SKILLS:
        loaded_skills = []
        for skill_id in state.ACTIVE_SKILLS:
            try:
                with open(f"assets/skills/{skill_id}.yaml", "r", encoding="utf-8") as f:
                    data = yaml.safe_load(f)
                    if data and "prompt" in data:
                        skill_name = data.get("name", skill_id)
                        loaded_skills.append(f"- {skill_name}: {data['prompt'].strip()}")
            except Exception: pass
        if loaded_skills:
            skills_block = "### CHARACTER ABILITIES ###\n" + "\n".join(loaded_skills)

    system_prompt = build_system_prompt(char_name, char_persona, rules_block, skills_block, context)
    # Inject modules right after rules if present
    if modules_block:
        system_prompt = system_prompt.replace("### THE PLAYER CHARACTER ###", f"{modules_block}\n\n### THE PLAYER CHARACTER ###")
    
    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": prompt}
    ]
    
    log.info(f"Initiating stream for char {char_id} in world {world_id}")
    full_content = ""
    total_tokens = 0

    try:
        if state.CURRENT_MODEL.startswith("deepseek-"):
            active_client = ds_client
        elif state.CURRENT_MODEL.startswith("groq/") or any(x in state.CURRENT_MODEL.lower() for x in ["llama-", "mixtral-", "gemma-"]) and not "/" in state.CURRENT_MODEL:
            active_client = groq_client
        else:
            active_client = client # Default to OpenRouter
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
