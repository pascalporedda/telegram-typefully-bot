#!/bin/bash

# Create voice-notes directory if it doesn't exist
mkdir -p voice-notes

# Create empty SQLite database file if it doesn't exist
touch bot.db

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "Creating .env file..."
    cat > .env << EOL
# Required
TELOXIDE_TOKEN=your_telegram_bot_token
OPENAI_API_KEY=your_openai_api_key

# Optional
RUST_LOG=info
EOL
    echo "Please edit .env file with your API keys"
fi

# Set proper permissions
chmod 644 bot.db
chmod 755 voice-notes

echo "Setup complete! Make sure to:"
echo "1. Edit .env with your API keys"
echo "2. Run 'docker compose up -d' to start the bot" 