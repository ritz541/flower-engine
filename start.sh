#!/bin/bash

# --- Color Definitions (Rose & Pink Palette) ---
PINK='\033[38;5;211m'
ROSE='\033[38;5;218m'
DEEP_ROSE='\033[38;5;161m'
GRAY='\033[38;5;240m'
NC='\033[0m' # No Color

# --- Cleanup ---
cleanup() {
    echo -e "\n\n${PINK} 🌸 The petals fold. Closing the book...${NC}"
    # Kill the background python process and any other background jobs
    kill $(jobs -p) 2>/dev/null
    exit
}
trap cleanup SIGINT SIGTERM

clear

# --- Pink Floral Banner ---
echo -e "${PINK}"
echo "      :::::::::: :::        ::::::::  :::       ::: :::::::::: ::::::::: "
echo "     :+:        :+:       :+:    :+: :+:       :+: :+:        :+:    :+: "
echo "    +:+        +:+       +:+    +:+ +:+       +:+ +:+        +:+    +:+  "
echo "   :#::+::#   +#+       +#+    +:+ +#+  +:+  +#+ +#::+::#   +#++:++#:    "
echo "  +#+        +#+       +#+    +#+ +#+ +#+#+ +#+ +#+        +#+    +#+    "
echo " #+#        #+#       #+#    #+#  #+#+# #+#+#  #+#        #+#    #+#     "
echo "###        ########## ########    ###   ###   ########## ###    ###      "
echo -e "${NC}"

echo -e "  ${ROSE}— Awakening the Narrator —${NC}\n"

# --- Backend Startup ---
# Ensure we are in the project root
export OPENAI_API_KEY="sk-or-dummy-key"
venv/bin/python -m uvicorn engine.main:app --host 0.0.0.0 --port 8000 --log-level error > /dev/null 2>&1 &
BACKEND_PID=$!

# Wait loop with flavor text
READY=0
for i in {1..40}; do
    if curl -s http://127.0.0.1:8000/ > /dev/null; then
        READY=1
        break
    fi
    
    # Cycle through roleplay-themed messages
    case $((i % 4)) in
        0) msg="Summoning characters..." ;;
        1) msg="Weaving the world..."    ;;
        2) msg="Sharpening the pen..."   ;;
        3) msg="Lighting the candles..." ;;
    esac
    
    # Progress bar made of petals (•)
    bar=""
    for ((j=0; j<i; j++)); do bar="${bar}•"; done
    
    echo -ne "\r  ${GRAY}✦ $msg ${NC}${ROSE}[$bar]${NC}   "
    sleep 0.4
done

# Check if backend actually stayed alive
if ps -p $BACKEND_PID > /dev/null && curl -s http://127.0.0.1:8000/ > /dev/null; then
    READY=1
fi

if [ $READY -eq 1 ]; then
    echo -e "\n\n  ${PINK}✨ THE STAGE IS SET. THE NARRATOR ASCENDS.${NC}\n"
else
    echo -e "\n\n  ${NC}The Narrator vanished into the void (Backend failed to start)."
    kill $BACKEND_PID 2>/dev/null
    exit 1
fi

# --- Frontend Launch ---
echo -e "  ${ROSE}✦ Entering the story...${NC}"
cd tui

# Use the pre-compiled release binary if it exists, otherwise fall back to cargo run
if [ -f "target/release/tui" ]; then
    ./target/release/tui
else
    cargo run --quiet
fi

cleanup
