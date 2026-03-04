#!/bin/bash

# --- Color Definitions ---
PINK='\033[38;5;211m'
ROSE='\033[38;5;218m'
NC='\033[0m' # No Color

echo -e "${PINK}🌸 The Flower Roleplay Engine - Initial Setup${NC}"
echo -e "${ROSE}Preparing the soil for your narrative...${NC}
"

# 1. Check for Prerequisites
echo -ne "✦ Checking for Python 3.12+..."
if ! command -v python3 &> /dev/null; then
    echo -e "
  [ERROR] Python 3 is not installed."
    exit 1
fi
echo " Done."

echo -ne "✦ Checking for Rust/Cargo..."
if ! command -v cargo &> /dev/null; then
    echo -e "
  [ERROR] Cargo is not installed. Visit https://rustup.rs/ to install Rust."
    exit 1
fi
echo " Done."

# 2. Create Virtual Environment
echo -ne "✦ Creating virtual environment (venv)..."
python3 -m venv venv
echo " Done."

# 3. Install Dependencies
echo -e "✦ Installing Python dependencies (Optimized for CPU)..."
# First, force install the CPU version of torch to avoid 8GB CUDA downloads
venv/bin/pip install torch --index-url https://download.pytorch.org/whl/cpu --quiet
# Then, install the rest of the requirements
venv/bin/pip install -r requirements.txt --quiet
echo "  Dependencies installed."

# 4. Initialize Assets
echo -ne "✦ Initializing assets..."
if [ ! -d "assets" ]; then
    cp -r assets_example assets
    echo " (Created assets/ from templates)"
else
    echo " (assets/ already exists, skipping)"
fi

# 5. Configuration Setup
echo -ne "✦ Preparing config.yaml..."
if [ ! -f "config.yaml" ]; then
    cp config.yaml.example config.yaml
    echo " (Created config.yaml from example)"
    echo -e "  ${ROSE}[ACTION REQUIRED] Please edit config.yaml to add your API keys.${NC}"
else
    echo " (config.yaml already exists, skipping)"
fi

# 6. Set Permissions
chmod +x start.sh
chmod +x setup.sh

echo -e "
${PINK}✨ SETUP COMPLETE!${NC}"
echo -e "1. Edit ${ROSE}config.yaml${NC} with your keys."
echo -e "2. Run ${ROSE}./start.sh${NC} to begin your journey."
