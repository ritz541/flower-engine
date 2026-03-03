import time
import json
import asyncio
import uuid
import yaml
from fastapi import WebSocket
from openai import AsyncOpenAI
from engine.logger import log
from engine.config import (
    OPENAI_BASE_URL,
    OPENAI_API_KEY,
    DEEPSEEK_API_KEY,
    GROQ_API_KEY,
    GROQ_BASE_URL,
)
from engine.database import (
    char_manager,
    world_manager,
    msg_manager,
    session_manager,
    Message,
)
from engine.rag import rag_manager
from engine.prompt import build_system_prompt
from engine.utils import build_ws_payload
import engine.state as state

client = AsyncOpenAI(
    base_url=OPENAI_BASE_URL,
    api_key=OPENAI_API_KEY,
    default_headers={
        "HTTP-Referer": "https://github.com/theflower",
        "X-Title": "The Flower Roleplay Engine",
    },
)

ds_client = AsyncOpenAI(base_url="https://api.deepseek.com", api_key=DEEPSEEK_API_KEY)

groq_client = AsyncOpenAI(base_url=GROQ_BASE_URL, api_key=GROQ_API_KEY)


async def stream_chat_response(
    ws: WebSocket,
    prompt: str,
    context: str,
    world_id: str,
    char_id: str,
    session_id: str = "",
):
    character = char_manager.get_character(char_id)
    char_name = character.name if character else "a wanderer"
    char_persona = character.persona if character else "A mysterious figure."

    history_count = 0
    if session_id:
        recent_msgs = msg_manager.get_messages(char_id, session_id, limit=2)
        history_count = len(recent_msgs)

    world = world_manager.get_world(world_id)
    world_scene = world.scene if world and world.scene else ""
    world_system_prompt = world.system_prompt if world and world.system_prompt else ""

    # Add scene to context only on first message
    if world_scene and history_count <= 1:
        context = f"--- SCENE ---\n{world_scene}\n\n{context}"

    # Note: world_system_prompt is now passed directly to build_system_prompt, not added to context

    rules_block = ""
    if state.ACTIVE_RULES:
        loaded_texts = []
        for rule_id in state.ACTIVE_RULES:
            try:
                with open(f"assets/rules/{rule_id}.yaml", "r", encoding="utf-8") as f:
                    data = yaml.safe_load(f)
                    if data and "prompt" in data:
                        loaded_texts.append(data["prompt"].strip())
            except Exception:
                pass
        if loaded_texts:
            rules_block = "### UNIVERSAL LAWS ###\n" + "\n\n".join(loaded_texts)

    system_prompt = build_system_prompt(
        char_name, char_persona, rules_block, world_system_prompt, context
    )

    history_messages = []
    if session_id:
        # Get last 11 messages (about 5 exchanges) to provide more context
        # We fetch 11 because the current prompt is already in the DB
        all_recent = msg_manager.get_messages(char_id, session_id, limit=11)
        
        # Filter out the current prompt to avoid double-entry in the messages list
        # The current prompt is the last one in all_recent
        if all_recent and all_recent[-1].content == prompt and all_recent[-1].role == "user":
            recent_msgs = all_recent[:-1]
        else:
            recent_msgs = all_recent
            
        # Take only the last 10 messages from what remains (5 full exchanges)
        recent_msgs = recent_msgs[-10:]
        
        for msg in recent_msgs:
            history_messages.append({"role": msg.role, "content": msg.content})

    # Build messages: system prompt FIRST, then history, then current prompt
    messages = [{"role": "system", "content": system_prompt}]
    messages.extend(history_messages)
    messages.append({"role": "user", "content": prompt})

    log.info(f"Initiating stream for char {char_id} in world {world_id}")
    full_content = ""
    total_tokens = 0

    try:
        if state.CURRENT_MODEL.startswith("deepseek-"):
            active_client = ds_client
            log.info(f"Using DeepSeek official client for {state.CURRENT_MODEL}")
        elif (
            state.CURRENT_MODEL.startswith("groq/")
            or any(
                x in state.CURRENT_MODEL.lower()
                for x in ["llama-", "mixtral-", "gemma-"]
            )
            and not "/" in state.CURRENT_MODEL
        ):
            active_client = groq_client
            log.info(f"Using Groq client for {state.CURRENT_MODEL}")
        else:
            active_client = client
            log.info(f"Using OpenRouter client for {state.CURRENT_MODEL}")

        response = await active_client.chat.completions.create(
            model=state.CURRENT_MODEL, messages=messages, stream=True
        )

        start_time = None
        async for chunk in response:
            if not start_time:
                start_time = time.time()
            delta = (
                chunk.choices[0].delta.content
                if chunk.choices and chunk.choices[0].delta
                else None
            )
            if delta:
                full_content += delta
                total_tokens += 1
                elapsed = time.time() - start_time
                tps = total_tokens / elapsed if elapsed > 0 else 0.0
                metadata = {
                    "model": state.CURRENT_MODEL,
                    "tokens_per_second": round(tps, 2),
                    "world_id": world_id,
                }
                await ws.send_text(build_ws_payload("chat_chunk", delta, metadata))

    except asyncio.CancelledError:
        log.info(f"Stream cancelled after {total_tokens} tokens.")
    except Exception as e:
        log.error(f"Error during streaming: {e}")
        await ws.send_text(build_ws_payload("error", str(e)))
        return
    finally:
        await ws.send_text(
            build_ws_payload("chat_end", "", {"total_tokens": total_tokens})
        )

    if full_content:
        # Save assistant response only (user message already saved in main.py)
        msg_manager.add_message(
            Message(
                role="assistant",
                content=full_content,
                character_id=char_id,
                session_id=session_id,
            )
        )

        memory_key = f"{char_id}_{session_id}" if session_id else char_id
        rag_manager.add_memory(
            memory_key, str(uuid.uuid4()), f"User: {prompt}\nAI: {full_content}"
        )

        if session_id:
            session_manager.touch(session_id)
            sess = session_manager.get_session(session_id)
            if sess and sess.title in ("", "New session"):
                session_manager.update_title(session_id, prompt)
