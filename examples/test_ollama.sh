#!/bin/bash

# Test script for Ollama integration with Commitor
# This script demonstrates how to use Commitor with Ollama

set -e

echo "🦙 Testing Commitor with Ollama"
echo "================================"

# Check if Ollama is running
echo "1. Checking Ollama availability..."
cargo run -- check-ollama

if [ $? -ne 0 ]; then
    echo "❌ Ollama is not available. Please start Ollama first:"
    echo "   ollama serve"
    exit 1
fi

# List available models
echo -e "\n2. Listing available Ollama models..."
cargo run -- models --provider ollama

# Check if we have any models
MODELS=$(cargo run -- models --provider ollama 2>/dev/null | grep -v "Available Ollama models:" | grep -v "No models found" | wc -l)

if [ "$MODELS" -eq 0 ]; then
    echo "❌ No models found. Please pull a model first:"
    echo "   ollama pull llama2"
    echo "   ollama pull codellama"
    echo "   ollama pull mistral"
    exit 1
fi

# Create a test repository if it doesn't exist
if [ ! -d "test_repo" ]; then
    echo -e "\n3. Creating test repository..."
    mkdir test_repo
    cd test_repo
    git init
    git config user.name "Test User"
    git config user.email "test@example.com"

    # Create a test file
    echo "# Test Project" > README.md
    echo "This is a test project for Commitor." >> README.md
    git add README.md
    git commit -m "initial commit"

    # Make some changes
    echo -e "\nAdding new functionality..." >> README.md
    echo "- Feature 1: User authentication" >> README.md
    echo "- Feature 2: Database integration" >> README.md
    git add README.md
else
    cd test_repo
fi

# Test commit message generation with different models
MODELS_TO_TEST=("llama2" "codellama" "mistral")

for model in "${MODELS_TO_TEST[@]}"; do
    echo -e "\n4. Testing with model: $model"
    echo "-----------------------------------"

    # Check if model is available
    if cargo run -- models --provider ollama 2>/dev/null | grep -q "$model"; then
        echo "✓ Model $model is available"

        # Generate commit messages
        echo "Generating commit messages..."
        cargo run -- --provider ollama --model "$model" --count 3 generate --show-diff

        echo -e "\nPress Enter to continue or Ctrl+C to exit..."
        read -r
    else
        echo "⚠ Model $model not available, skipping..."
    fi
done

# Test auto-commit (with confirmation)
echo -e "\n5. Testing auto-commit functionality..."
echo "This will automatically commit with the first generated message."
echo "Continue? (y/N)"
read -r response

if [[ "$response" =~ ^[Yy]$ ]]; then
    # Make another change
    echo -e "\nBug fix: resolve authentication issue" >> README.md
    git add README.md

    # Auto-commit with the first suggestion
    cargo run -- --provider ollama --model "llama2" commit --auto-commit
else
    echo "Skipping auto-commit test."
fi

# Cleanup
cd ..
echo -e "\n✅ Ollama integration test completed!"
echo "To clean up the test repository, run: rm -rf test_repo"
