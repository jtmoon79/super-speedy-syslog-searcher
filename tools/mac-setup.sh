#!/usr/bin/env bash
#
# mac-setup.sh
#
# Steps to setup a new Mac environment for basic development.
# Useful for when I rent a Mac to check stuff in this project.

set -euo pipefail

cd "$(dirname "${0}")/.."

# Install Homebrew
if ! command -v brew &> /dev/null; then
    echo "Homebrew not found. Installing Homebrew..."
    (
        set -eo pipefail
        set -x
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    )
else
    echo "Homebrew is already installed."
fi

# install bash 5
if ! command -v bash &> /dev/null || [[ "$(bash -c 'echo $BASH_VERSION')" != *"5."* ]]; then
    echo "Bash 5 not found. Installing Bash 5..."
    (set -x; /opt/homebrew/bin/brew install -y bash)
else
    echo "Bash 5 is already installed."
fi

# install custom bash profile
(
    set -eo pipefail
    cd ~
    set -x
    curl --silent 'https://raw.githubusercontent.com/jtmoon79/dotfiles/master/install.sh' | bash --norc --noprofile
)
echo '
bash_paths_add ~/.local/bin /opt/homebrew/bin
' >> ~/.bashrc.local.post
bash_paths_add ~/.local/bin /opt/homebrew/bin

# install Python
if ! command -v python3 &> /dev/null; then
    echo "Python 3 not found. Installing Python 3..."
    (set -x; brew install -y python)
else
    echo "Python 3 is already installed."
fi

# create Pythen venv
(set -x; ./tools/venv-create.sh)

# install rust
if ! command -v cargo &> /dev/null; then
    echo "Rust not found. Installing Rust..."
    (
        set -eo pipefail
        set -x
        curl --silent https://sh.rustup.rs -sSf | sh -s -- -y
    )
else
    echo "Rust is already installed."
fi
echo '
bash_paths_add ~/.cargo/bin
' >> ~/.bashrc.local.post
bash_paths_add ~/.cargo/bin

# install Rust tools
(
    source "$HOME/.cargo/env"
    set -x
    rustup toolchain add nightly
    cargo install cargo-cross cross
)

# install comparison tools
(
    set -x
    brew install -y lnav
)
