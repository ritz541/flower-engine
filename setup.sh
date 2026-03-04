#!/bin/bash

# --- Color Definitions ---
PINK='\033[38;5;211m'
ROSE='\033[38;5;218m'
NC='\033[0m' # No Color

# --- Branding Banner ---
clear
echo -e "${PINK}"
echo "      :::::::::: :::        ::::::::  :::       ::: :::::::::: ::::::::: "
echo "     :+:        :+:       :+:    :+: :+:       :+: :+:        :+:    :+: "
echo "    +:+        +:+       +:+    +:+ +:+       +:+ +:+        +:+    +:+  "
echo "   :#::+::#   +#+       +#+    +:+ +#+  +:+  +#+ +#::+::#   +#++:++#:    "
echo "  +#+        +#+       +#+    +#+ +#+ +#+#+ +#+ +#+        +#+    +#+    "
echo " #+#        #+#       #+#    #+#  #+#+# #+#+#  #+#        #+#    #+#     "
echo "###        ########## ########    ###   ###   ########## ###    ###      "
echo -e "${NC}"

echo -e "  ${ROSE}— Preparing the Soil for Your Narrative —${NC}\n"

# 1. Check for Prerequisites
echo -ne "  ${ROSE}✦${NC} Checking for Python 3.12+..."
if ! command -v python3 &> /dev/null; then
    echo -e "\n  ${NC}[ERROR] Python 3 is not installed."
    exit 1
fi
echo " Done."

echo -ne "  ${ROSE}✦${NC} Checking for Rust/Cargo..."
if ! command -v cargo &> /dev/null; then
    echo -e "\n  ${NC}[!] Rust/Cargo not found."
    read -p "      Would you like to download the pre-compiled TUI binary instead? (y/n): " choice
    if [ "$choice" = "y" ] || [ "$choice" = "Y" ]; then
        echo -e "  ${ROSE}✦${NC} Downloading flower-tui-linux-x64..."
        mkdir -p tui/target/release
        curl -L "https://github.com/ritz541/flower-engine/releases/download/v1.0.0/flower-tui-linux-x64" -o tui/target/release/tui
        chmod +x tui/target/release/tui
        echo "      Done (Binary installed)."
    else
        echo -e "  ${NC}[ERROR] Rust is required to build from source. Visit https://rustup.rs/ to install."
        exit 1
    fi
else
    echo " Done."
fi

# 2. Create Virtual Environment
echo -ne "  ${ROSE}✦${NC} Creating virtual environment (venv)..."
python3 -m venv venv
echo " Done."

# 3. Install Dependencies
echo -e "  ${ROSE}✦${NC} Summoning dependencies (Optimized for CPU)..."
echo -e "     ${ROSE}This may take a moment while the petals bloom...${NC}"

# Start a simple spinner in the background
spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='|/-\'
    while [ "$(ps a | awk '{print $1}' | grep $pid)" ]; do
        local temp=${spinstr#?}
        printf "     [%c]  " "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b\b\b\b\b"
    done
    printf "    \b\b\b\b"
}

# Run pip in background
(
    venv/bin/pip install torch --index-url https://download.pytorch.org/whl/cpu --quiet
    venv/bin/pip install -r requirements.txt --quiet
) &

PIP_PID=$!
spinner $PIP_PID

echo -e "\n     ${ROSE}✓ Dependencies successfully gathered.${NC}"

# 4. Initialize Assets
echo -ne "  ${ROSE}✦${NC} Initializing assets..."
if [ ! -d "assets" ]; then
    cp -r assets_example assets
    echo " (Created assets/ from templates)"
else
    # Update system_rules if missing even if assets/ exists
    if [ ! -f "assets/system_rules.yaml" ] && [ -f "assets_example/system_rules.yaml" ]; then
        cp "assets_example/system_rules.yaml" "assets/system_rules.yaml"
    fi
    echo " (Already exists)"
fi

# 5. Configuration Setup
echo -ne "  ${ROSE}✦${NC} Preparing config.yaml..."
if [ ! -f "config.yaml" ]; then
    cp config.yaml.example config.yaml
    echo " (Created from example)"
else
    echo " (Already exists)"
fi

# 6. Set Permissions
chmod +x start.sh
chmod +x setup.sh

echo -e "\n  ${PINK}✨ THE SOIL IS READY. THE NARRATOR AWAITS.${NC}"
echo -e "  ${ROSE}1.${NC} Edit ${ROSE}config.yaml${NC} to add your API keys."
echo -e "  ${ROSE}2.${NC} Run ${ROSE}./start.sh${NC} to begin your story.\n"
