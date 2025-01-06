# Telegram Voice Note to Typefully Bot

A Telegram bot that transcribes voice notes and creates social media drafts in Typefully. Built with Rust, using OpenAI's Whisper for transcription and GPT-4o-mini for post processing.

## Features

- 🎙️ Voice note transcription using OpenAI's Whisper API
- ✍️ Automatic social media post generation
- 📝 Direct integration with Typefully for draft creation
- 🎁 5 minutes of free transcription
- 🔑 Support for custom OpenAI API keys
- 📊 Usage tracking and management

## Prerequisites

- Rust (latest stable version) OR Docker
- SQLite (only for non-Docker setup)
- A Telegram Bot Token (from [@BotFather](https://t.me/botfather))
- A Typefully API Key (from [Typefully Settings](https://typefully.com))
- An OpenAI API Key (optional for users, required for bot operator)

## Setup

### Using Docker (Recommended)

1. Clone the repository:
```bash
git clone https://github.com/yourusername/telegram-typefully-bot
cd telegram-typefully-bot
```

2. Run the setup script:
```bash
chmod +x setup.sh
./setup.sh
```

3. Edit the `.env` file with your API keys

4. Start the bot:
```bash
docker compose up -d
```

### Manual Setup

1. Clone the repository:
```bash
git clone https://github.com/yourusername/telegram-typefully-bot
cd telegram-typefully-bot
```

2. Create a `.env` file in the project root:
```env
TELOXIDE_TOKEN=your_telegram_bot_token
OPENAI_API_KEY=your_openai_api_key  # For free tier usage
```

3. Create the voice notes directory:
```bash
mkdir voice-notes
```

4. Build and run the bot:
```bash
cargo build --release
cargo run --release
```

## Usage

1. Start the bot with `/start`
2. Provide your Typefully API key when prompted
3. Send a voice note to the bot
4. The bot will:
   - Transcribe your voice note
   - Generate a social media post
   - Create a draft in your Typefully account

## Commands

- `/help` - Show available commands and usage instructions
- `/setapikey` - Set your own OpenAI API key (optional)
- `/settypefullykey` - Update your Typefully API key
- `/usage` - Check your remaining free transcription time
- `/deleteaccount` - Delete your account and data

## Free Usage

- Each user gets 5 minutes of free transcription
- After the free tier is exhausted, users need to provide their own OpenAI API key
- Usage is tracked per user to prevent abuse

## Development

The bot is built with:
- [Teloxide](https://github.com/teloxide/teloxide) for Telegram integration
- [SQLx](https://github.com/launchbadge/sqlx) for database operations
- [OpenAI API](https://platform.openai.com/) for transcription and post generation
- [Typefully API](https://typefully.com) for draft creation

## Database

The bot uses SQLite for data storage, with tables for:
- Users and their API keys
- Voice note usage tracking
- Deleted user records

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| TELOXIDE_TOKEN | Your Telegram Bot Token | Yes |
| OPENAI_API_KEY | OpenAI API Key for free tier usage | Yes |
| RUST_LOG | Log level (e.g., "info") | No |

## Docker Volumes

The Docker setup uses two mounted volumes:
- `./bot.db:/app/bot.db` - SQLite database file
- `./voice-notes:/app/voice-notes` - Temporary storage for voice notes

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the GPL-2.0 License - see the LICENSE file for details.

## Security Notes

- API keys are stored in the database
- Voice notes are automatically deleted after processing
- Usage statistics are retained even after account deletion to prevent abuse
- Users can provide their own OpenAI API keys for unlimited usage
