import os
import chromadb
from chromadb.utils import embedding_functions
from chromadb.api.models.Collection import Collection
from typing import List, Dict, Any, Tuple

from engine.logger import log

class RagManager:
    def __init__(self, db_path: str = "./chroma_db"):
        self.db_path = db_path
        
        # Ensure directory exists for Chroma persistent db
        os.makedirs(db_path, exist_ok=True)
        
        # Persistent client stores data to disk
        self.client = chromadb.PersistentClient(path=db_path)
        
        # Use sentence-transformers for local, fast embeddings
        model_name = "all-MiniLM-L6-v2"
        log.info(f"Initializing SentenceTransformerEmbeddingFunction with model: {model_name}")
        self.embedding_function = embedding_functions.SentenceTransformerEmbeddingFunction(
            model_name=model_name
        )
        
        # We will use one main collection for lore and another for memory
        self.collection_name = "world_lore"
        self._collection = None
        self.memory_collection_name = "session_memory"
        self._memory_collection = None

    @property
    def collection(self) -> Collection:
        if self._collection is None:
            log.info(f"Getting or creating collection: {self.collection_name}")
            self._collection = self.client.get_or_create_collection(
                name=self.collection_name,
                embedding_function=self.embedding_function,
                metadata={"hnsw:space": "cosine"}
            )
        return self._collection
        
    @property
    def memory_collection(self) -> Collection:
        if self._memory_collection is None:
            log.info(f"Getting or creating memory collection: {self.memory_collection_name}")
            self._memory_collection = self.client.get_or_create_collection(
                name=self.memory_collection_name,
                embedding_function=self.embedding_function,
                metadata={"hnsw:space": "cosine"}
            )
        return self._memory_collection

    def add_lore(self, world_id: str, lore_id: str, text: str, metadata: Dict[str, Any] = None):
        """Add a document to the lore collection for a specific world."""
        meta = metadata or {}
        meta["world_id"] = world_id  # Ensure world filtering works
        
        log.info(f"Adding lore {lore_id} to world {world_id}")
        self.collection.upsert(
            ids=[f"{world_id}_{lore_id}"],
            documents=[text],
            metadatas=[meta]
        )

    def query_lore(self, world_id: str, query: str, n_results: int = 3, max_chars: int = 1000) -> Tuple[List[str], bool]:
        """Query lore specifically for the given world. Returns (results, context_warning)."""
        log.debug(f"Querying lore for world {world_id}: '{query}'")
        try:
            results = self.collection.query(
                query_texts=[query],
                n_results=n_results,
                where={"world_id": world_id} # Filter by world ID
            )
            
            # results["documents"] is a list of lists (one list per query)
            if results["documents"] and results["documents"][0]:
                docs = results["documents"][0]
                
                # Check for context window bloat
                total_chars = sum(len(d) for d in docs)
                context_warning = total_chars > max_chars
                
                return docs, context_warning
            return [], False
        except Exception as e:
            log.error(f"Error querying lore: {e}")
            return [], False

    def add_memory(self, session_id: str, memory_id: str, text: str):
        """Add a recent exchange to the session memory collection."""
        log.info(f"Adding memory {memory_id} to session {session_id}")
        self.memory_collection.upsert(
            ids=[f"{session_id}_{memory_id}"],
            documents=[text],
            metadatas=[{"session_id": session_id}]
        )
        
    def query_memory(self, session_id: str, query: str, n_results: int = 3, max_chars: int = 1500) -> Tuple[List[str], bool]:
        """Query memory for the given session. Returns (results, context_warning)."""
        log.debug(f"Querying memory for session {session_id}: '{query}'")
        try:
            results = self.memory_collection.query(
                query_texts=[query],
                n_results=n_results,
                where={"session_id": session_id} 
            )
            
            if results["documents"] and results["documents"][0]:
                docs = results["documents"][0]
                total_chars = sum(len(d) for d in docs)
                context_warning = total_chars > max_chars
                return docs, context_warning
            return [], False
        except Exception as e:
            log.error(f"Error querying memory: {e}")
            return [], False

    def delete_session_memory(self, session_id: str):
        """Physically delete all vector embeddings for a specific session."""
        log.info(f"Deleting vector memory for session {session_id}")
        try:
            self.memory_collection.delete(where={"session_id": session_id})
        except Exception as e:
            log.error(f"Failed to delete vector memory: {e}")

# Expose a singleton-like instance or let main.py instantiate
rag_manager = RagManager()
