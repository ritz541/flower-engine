import json
import os
from engine.config import MODEL_NAME
from engine.utils import load_yaml_assets

PERSIST_FILE = "persist.json"


def save_state():
    data = {
        "world_id": ACTIVE_WORLD_ID,
        "character_id": ACTIVE_CHARACTER_ID,
        "session_id": ACTIVE_SESSION_ID,
        "model": CURRENT_MODEL,
        "model_confirmed": MODEL_CONFIRMED,
        "rules": ACTIVE_RULES,
    }
    with open(PERSIST_FILE, "w") as f:
        json.dump(data, f)


def load_persisted_state():
    global \
        ACTIVE_WORLD_ID, \
        ACTIVE_CHARACTER_ID, \
        CURRENT_MODEL, \
        MODEL_CONFIRMED, \
        ACTIVE_RULES
    if os.path.exists(PERSIST_FILE):
        try:
            with open(PERSIST_FILE, "r") as f:
                data = json.load(f)
                ACTIVE_WORLD_ID = data.get("world_id", "")
                ACTIVE_CHARACTER_ID = data.get("character_id", "")
                CURRENT_MODEL = data.get("model", MODEL_NAME)
                MODEL_CONFIRMED = data.get("model_confirmed", False)
                ACTIVE_RULES = data.get("rules", [])
        except:
            pass


ACTIVE_WORLD_ID = ""
ACTIVE_CHARACTER_ID = ""
ACTIVE_SESSION_ID = ""
CURRENT_MODEL = MODEL_NAME
MODEL_CONFIRMED = False

ACTIVE_RULES = []

available_worlds = [
    {"id": d["id"], "name": d.get("name", d["id"])}
    for d in load_yaml_assets("assets/worlds/*.yaml")
]
available_characters = [
    {"id": d["id"], "name": d.get("name", d["id"])}
    for d in load_yaml_assets("assets/characters/*.yaml")
]
available_rules = [
    {"id": d["id"], "name": d.get("name", d["id"])}
    for d in load_yaml_assets("assets/rules/*.yaml")
]
AVAILABLE_MODELS = []

load_persisted_state()
