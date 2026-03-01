import json
import yaml
import glob
from typing import Dict, Any, List
from engine.logger import log

def build_ws_payload(event: str, content: str, metadata: Dict[str, Any] = None) -> str:
    return json.dumps({
        "event": event,
        "payload": {
            "content": content,
            "metadata": metadata or {}
        }
    })

def load_yaml_assets(pattern: str) -> List[Dict[str, Any]]:
    assets = []
    for filepath in sorted(glob.glob(pattern)):
        try:
            with open(filepath, "r", encoding="utf-8") as f:
                data = yaml.safe_load(f)
                if data and "id" in data:
                    assets.append(data)
        except Exception as e:
            log.error(f"Failed to load asset {filepath}: {e}")
    return assets
