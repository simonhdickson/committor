#!/bin/bash

# Commitor Installation Script
# This script installs commitor and sets up the environment

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check system requirements
check_requirements() {
    print_status "Checking system requirements..."

    # Check for Rust
    if ! command_exists rustc; then
        print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi

    # Check Rust version
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    print_success "Rust $RUST_VERSION found"

    # Check for Cargo
    if ! command_exists cargo; then
        print_error "Cargo is not installed. Please install Rust toolchain from https://rustup.rs/"
        exit 1
    fi

    # Check for Git
    if ! command_exists git; then
        print_error "Git is not installed. Please install Git first."
        exit 1
    fi

    GIT_VERSION=$(git --version | cut -d' ' -f3)
    print_success "Git $GIT_VERSION found"
}

# Function to install commitor
install_commitor() {
    print_status "Installing commitor..."

    if [ -d "commitor" ]; then
        print_status "Found existing commitor directory, updating..."
        cd commitor
        git pull origin main || print_warning "Could not update repository"
    else
        print_status "Cloning commitor repository..."
        git clone https://github.com/simonhdickson/commitor.git
        cd commitor
    fi

    print_status "Building commitor..."
    cargo build --release

    if [ $? -eq 0 ]; then
        print_success "Commitor built successfully!"
    else
        print_error "Failed to build commitor"
        exit 1
    fi
}

# Function to install to system
install_to_system() {
    print_status "Installing commitor to system..."

    # Install using cargo
    cargo install --path . --force

    if [ $? -eq 0 ]; then
        print_success "Commitor installed to cargo bin directory"
    else
        print_error "Failed to install commitor"
        exit 1
    fi

    # Check if cargo bin is in PATH
    if ! echo "$PATH" | grep -q "$HOME/.cargo/bin"; then
        print_warning "Cargo bin directory is not in PATH"
        print_status "Add the following line to your shell profile (.bashrc, .zshrc, etc.):"
        echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    fi
}

# Function to setup API key
setup_api_key() {
    print_status "Setting up OpenAI API key..."

    if [ -n "$OPENAI_API_KEY" ]; then
        print_success "OpenAI API key is already set in environment"
        return
    fi

    echo
    echo "To use commitor, you need an OpenAI API key."
    echo "You can get one from: https://platform.openai.com/api-keys"
    echo
    read -p "Do you want to set your API key now? (y/N): " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        read -p "Enter your OpenAI API key: " -s API_KEY
        echo

        if [ -n "$API_KEY" ]; then
            # Determine which shell config file to use
            if [ -n "$ZSH_VERSION" ]; then
                SHELL_CONFIG="$HOME/.zshrc"
            elif [ -n "$BASH_VERSION" ]; then
                SHELL_CONFIG="$HOME/.bashrc"
            else
                SHELL_CONFIG="$HOME/.profile"
            fi

            echo "export OPENAI_API_KEY=\"$API_KEY\"" >> "$SHELL_CONFIG"
            print_success "API key added to $SHELL_CONFIG"
            print_status "Please restart your terminal or run: source $SHELL_CONFIG"
        else
            print_warning "No API key provided. You can set it later with:"
            echo "export OPENAI_API_KEY=\"your-api-key-here\""
        fi
    else
        print_status "You can set your API key later by adding this to your shell profile:"
        echo "export OPENAI_API_KEY=\"your-api-key-here\""
    fi
}

# Function to verify installation
verify_installation() {
    print_status "Verifying installation..."

    if command_exists commitor; then
        COMMITOR_VERSION=$(commitor --version 2>/dev/null || echo "unknown")
        print_success "Commitor is installed and accessible: $COMMITOR_VERSION"

        # Test basic functionality
        print_status "Testing basic functionality..."
        if commitor --help >/dev/null 2>&1; then
            print_success "Commitor help command works"
        else
            print_warning "Commitor help command failed"
        fi
    else
        print_error "Commitor is not accessible. Check your PATH."
        exit 1
    fi
}

# Function to show usage examples
show_usage() {
    print_success "Installation complete! Here are some usage examples:"
    echo
    echo "Basic usage:"
    echo "  commitor generate                 # Generate commit messages for staged changes"
    echo "  commitor commit                   # Generate and commit in one step"
    echo "  commitor diff                     # Show current staged diff"
    echo
    echo "Advanced usage:"
    echo "  commitor generate --count 5       # Generate 5 options"
    echo "  commitor commit --auto-commit     # Auto-commit with first suggestion"
    echo "  commitor generate --show-diff     # Show diff before generating"
    echo "  commitor generate --model gpt-3.5-turbo  # Use different model"
    echo
    echo "For more information, run: commitor --help"
    echo
    print_status "Make sure to set your OPENAI_API_KEY environment variable!"
}

# Main installation process
main() {
    echo "🚀 Commitor Installation Script"
    echo "================================"
    echo

    check_requirements
    install_commitor
    install_to_system
    setup_api_key
    verify_installation
    show_usage

    print_success "Installation completed successfully! 🎉"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  --help, -h     Show this help message"
            echo "  --no-api-key   Skip API key setup"
            echo
            exit 0
            ;;
        --no-api-key)
            SKIP_API_KEY=1
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Override setup_api_key if --no-api-key is specified
if [ "$SKIP_API_KEY" = "1" ]; then
    setup_api_key() {
        print_status "Skipping API key setup (--no-api-key specified)"
    }
fi

# Run main installation
main
