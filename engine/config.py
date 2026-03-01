import os
import yaml
from engine.logger import log

CONFIG = {}
try:
    with open("config.yaml", "r") as f:
        CONFIG = yaml.safe_load(f)
except Exception as e:
    log.error(f"Failed to load config.yaml: {e}")

MODEL_NAME = CONFIG.get("default_model", os.getenv("MODEL_NAME", "google/gemini-2.0-pro-exp-02-05:free"))

SUPPORTED_MODELS = CONFIG.get("supported_models", [
    "google/gemini-2.0-pro-exp-02-05:free",
    "openai/gpt-4o-mini",
    "anthropic/claude-3-haiku",
])

OPENAI_BASE_URL = CONFIG.get("openai_base_url", os.getenv("OPENAI_BASE_URL", "https://openrouter.ai/api/v1"))
OPENAI_API_KEY = CONFIG.get("openai_api_key", os.getenv("OPENAI_API_KEY", "dummy_key_if_local"))
DEEPSEEK_API_KEY = CONFIG.get("deepseek_api_key", os.getenv("DEEPSEEK_API_KEY", "dummy_deepseek_key"))
GROQ_API_KEY = CONFIG.get("groq_api_key", os.getenv("GROQ_API_KEY", "dummy_groq_key"))
GROQ_BASE_URL = "https://api.groq.com/openai/v1"
