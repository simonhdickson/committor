#!/bin/bash

# Demo script for Commitor with Ollama integration
# This script demonstrates the new multi-provider capabilities

set -e

echo "🚀 Commitor Multi-Provider Demo"
echo "==============================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "Not in a git repository. Please run this script from a git repository."
    exit 1
fi

print_step "1. Checking Ollama availability..."
if cargo run --release -- check-ollama > /dev/null 2>&1; then
    print_success "Ollama is available!"
else
    print_warning "Ollama is not available. Some demos will be skipped."
    print_info "To install Ollama: https://ollama.ai"
    print_info "To start Ollama: ollama serve"
    OLLAMA_AVAILABLE=false
fi

print_step "2. Checking OpenAI configuration..."
if [ -n "$OPENAI_API_KEY" ]; then
    print_success "OpenAI API key is configured!"
    OPENAI_AVAILABLE=true
else
    print_warning "OPENAI_API_KEY not set. OpenAI demos will be skipped."
    print_info "To use OpenAI: export OPENAI_API_KEY=\"your-key-here\""
    OPENAI_AVAILABLE=false
fi

if [ "$OLLAMA_AVAILABLE" = false ] && [ "$OPENAI_AVAILABLE" = false ]; then
    print_error "Neither Ollama nor OpenAI is available. Please set up at least one provider."
    exit 1
fi

print_step "3. Listing available models..."

if [ "$OPENAI_AVAILABLE" = true ]; then
    echo
    print_info "OpenAI models:"
    cargo run --release -- models --provider openai
fi

if [ "$OLLAMA_AVAILABLE" != false ]; then
    echo
    print_info "Ollama models:"
    cargo run --release -- models --provider ollama
fi

# Create some demo changes if none exist
if ! git diff --cached --quiet 2>/dev/null; then
    print_success "Found existing staged changes!"
else
    print_step "4. Creating demo changes..."

    # Create a demo file with some changes
    echo "# Demo Project" > demo_file.md
    echo "This is a demo file for testing Commitor." >> demo_file.md
    echo "" >> demo_file.md
    echo "## Features" >> demo_file.md
    echo "- Multi-provider AI support" >> demo_file.md
    echo "- OpenAI integration" >> demo_file.md
    echo "- Ollama local processing" >> demo_file.md

    git add demo_file.md
    print_success "Created and staged demo changes!"
fi

print_step "5. Showing current diff..."
cargo run --release -- diff

echo
print_step "6. Running AI provider demos..."

# Demo with OpenAI if available
if [ "$OPENAI_AVAILABLE" = true ]; then
    echo
    print_info "Demo: OpenAI with GPT-4"
    echo "Command: commitor --provider openai --model gpt-4 --count 3 generate"
    cargo run --release -- --provider openai --model gpt-4 --count 3 generate

    echo
    print_info "Demo: OpenAI with GPT-3.5-turbo (faster)"
    echo "Command: commitor --provider openai --model gpt-3.5-turbo --count 2 generate"
    cargo run --release -- --provider openai --model gpt-3.5-turbo --count 2 generate
fi

# Demo with Ollama if available
if [ "$OLLAMA_AVAILABLE" != false ]; then
    echo
    print_info "Demo: Ollama with default model"
    echo "Command: commitor --provider ollama --model llama2 --count 3 generate"
    if cargo run --release -- --provider ollama --model llama2 --count 3 generate 2>/dev/null; then
        print_success "Ollama demo completed!"
    else
        print_warning "Ollama demo failed - you might need to pull the llama2 model:"
        print_info "Run: ollama pull llama2"
    fi

    # Try with codellama if available
    echo
    print_info "Demo: Ollama with CodeLlama (better for code)"
    echo "Command: commitor --provider ollama --model codellama --count 2 generate"
    if cargo run --release -- --provider ollama --model codellama --count 2 generate 2>/dev/null; then
        print_success "CodeLlama demo completed!"
    else
        print_warning "CodeLlama not available. To install:"
        print_info "Run: ollama pull codellama"
    fi
fi

echo
print_step "7. Performance comparison demo..."

if [ "$OPENAI_AVAILABLE" = true ] && [ "$OLLAMA_AVAILABLE" != false ]; then
    print_info "Comparing response times between providers..."

    echo
    print_info "Timing OpenAI GPT-3.5-turbo..."
    time cargo run --release -- --provider openai --model gpt-3.5-turbo --count 1 generate > /dev/null 2>&1 || true

    echo
    print_info "Timing Ollama llama2..."
    time cargo run --release -- --provider ollama --model llama2 --count 1 generate > /dev/null 2>&1 || true
fi

echo
print_step "8. Advanced features demo..."

if [ "$OLLAMA_AVAILABLE" != false ]; then
    print_info "Demo: Custom Ollama URL and timeout"
    echo "Command: commitor --provider ollama --ollama-url http://localhost:11434 --ollama-timeout 45 --model llama2 generate"
    cargo run --release -- --provider ollama --ollama-url http://localhost:11434 --ollama-timeout 45 --model llama2 generate 2>/dev/null || print_warning "Custom config demo failed"
fi

print_info "Demo: Show diff before generation"
echo "Command: commitor --provider [any] --show-diff generate"
if [ "$OPENAI_AVAILABLE" = true ]; then
    cargo run --release -- --provider openai --model gpt-3.5-turbo --show-diff --count 1 generate 2>/dev/null || true
elif [ "$OLLAMA_AVAILABLE" != false ]; then
    cargo run --release -- --provider ollama --model llama2 --show-diff --count 1 generate 2>/dev/null || true
fi

echo
print_step "9. Cleanup..."
echo "Do you want to remove the demo file? (y/N)"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    git reset demo_file.md 2>/dev/null || true
    rm -f demo_file.md
    print_success "Demo file removed!"
else
    print_info "Demo file kept. You can commit it or remove it manually."
fi

echo
print_success "Demo completed! 🎉"
echo
print_info "Next steps:"
echo "  1. Set up your preferred AI provider (OpenAI or Ollama)"
echo "  2. Stage some real changes: git add <files>"
echo "  3. Generate commit messages: commitor generate"
echo "  4. Or auto-commit: commitor commit --auto-commit"
echo
print_info "For more help:"
echo "  commitor --help"
echo "  commitor models --provider <openai|ollama>"
echo "  commitor check-ollama (for Ollama troubleshooting)"
echo
print_info "Documentation:"
echo "  README.md - General usage"
echo "  OLLAMA_GUIDE.md - Detailed Ollama setup and usage"
