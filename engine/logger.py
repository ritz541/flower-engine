import logging
import sys

def setup_logger():
    logger = logging.getLogger("engine")
    logger.setLevel(logging.DEBUG)
    
    # File handler
    file_handler = logging.FileHandler("engine.log")
    file_handler.setLevel(logging.DEBUG)
    
    # Formatter
    formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    file_handler.setFormatter(formatter)
    
    # Add handler to logger
    # Only adding FileHandler so stdout stays clean for TUI
    if not logger.handlers:
        logger.addHandler(file_handler)
        
    return logger

log = setup_logger()
