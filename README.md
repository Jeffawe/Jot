# jot

> Your digital memory. Search everything you've copied or typed, instantly.

> ⚠️ **WARNING: Heavy Development**  
> jot is currently in active development and not ready for production use. Features are incomplete, APIs will change, and bugs are expected. Use at your own risk.

**jot** is a fully local, privacy-first AI assistant that remembers your clipboard history and terminal commands. Ask questions in natural language and get instant answers, no scrolling, no searching, just results.



## ✨ Features

- 🔍 **Semantic Search** - Find things by meaning, not just keywords
- 🤖 **Local AI** - Explain commands, summarize activity, answer questions
- 🔒 **100% Private** - Everything stays on your machine, encrypted
- ⚡ **Lightning Fast** - Search 10,000+ items in milliseconds
- 🎨 **Dual Interface** - Beautiful GUI or blazing-fast CLI
- 🧠 **Context Aware** - Understands what you're looking for

## 🎯 Use Cases

```bash
# Instead of scrolling through terminal history
jot "ssh command for staging server"
→ ssh user@staging.example.com -i ~/.ssh/key.pem

# Find that email you copied hours ago
jot "email address from this morning"
→ john.doe@example.com

# Get explanations
jot explain "docker run -p 3000:3000 --rm myapp"
→ This command runs a Docker container...

# Summarize your day
jot "what did I work on today?"
→ You worked on: SSH configurations, Docker deployments...
```

## 🚀 Quick Start

### Installation

**macOS** (Currently supported)

```bash
# Clone the repository
git clone https://github.com/Jeffawe/Jot
cd jot

# Build from source
cargo build --release

# Install
cargo install --path .
```

### First Run

```bash
# Start monitoring (runs in background)
jot monitor

# Search your history
jot search "ssh"

# Ask questions
jot ask "what was that git command from yesterday?"
```

## 📖 How It Works

### Phase 1: Semantic Search (Current)

1. **Monitors** your clipboard and terminal silently in the background
2. **Stores** everything locally in an encrypted SQLite database
3. **Indexes** content using embedding models for semantic search
4. **Searches** using natural language to find what you need

### Phase 2: Local AI (Coming Soon)

- **TinyLlama 1.1B** for explanations and summaries
- **Context-aware** responses based on your history
- **Privacy-first** - model runs entirely on your machine

## 💻 Usage

### CLI Commands

```bash
# Start clipboard & terminal monitoring
jot monitor

# Search with natural language
jot search "docker command with port mapping"
jot search "email from sarah" --limit 10

# Explain a command (Phase 2)
jot explain "ssh -L 8080:localhost:80 user@server"

# View recent history
jot recent
jot recent --count 20

# Clear history
jot clear --before "7 days ago"
jot clear --all

# Configure
jot config edit
jot config show
```

### GUI Mode

```bash
# Launch the desktop app
jot gui

# Or use global hotkey (configurable)
Cmd+Shift+J  # macOS default
```

## ⚙️ Configuration

Configuration file: `~/.jot/config.toml`

```toml
[monitoring]
clipboard_enabled = true
terminal_enabled = true
poll_interval_ms = 500

[storage]
retention_days = 30
max_db_size_mb = 500
db_path = "~/.jot/history.db"

[search]
embedding_model = "all-MiniLM-L6-v2"
similarity_threshold = 0.7
max_results = 10

[privacy]
# Exclude sensitive apps (password managers, etc.)
exclude_apps = ["1Password", "Bitwarden", "Keychain Access"]

# Exclude file patterns
exclude_patterns = ["*.env", "*.key", "*.pem", "*.password"]

# Exclude folders
exclude_folders = ["/secrets/", "~/.ssh/"]

# Auto-detect and exclude sensitive data
auto_detect_passwords = true
auto_detect_api_keys = true
auto_detect_credit_cards = true

[ui]
hotkey = "Cmd+Shift+J"
theme = "system"  # "dark" | "light" | "system"
```

## 🔒 Privacy & Security

**jot** is built privacy-first:

- ✅ **100% Local** - No data ever leaves your machine
- ✅ **Encrypted Storage** - Database encrypted with sqlcipher
- ✅ **No Telemetry** - Zero analytics or tracking
- ✅ **Configurable Exclusions** - Block apps, files, or patterns
- ✅ **Open Source** - Fully auditable code

### Exclusion System

Protect sensitive data automatically:

```bash
# Exclude specific apps
jot config exclude-app "1Password"

# Exclude file patterns
jot config exclude-pattern "*.env"

# Exclude folders
jot config exclude-folder "~/Documents/Private"

# View exclusions
jot config list-exclusions
```

## 🛠️ Tech Stack

- **Language**: Rust 🦀
- **Storage**: SQLite with sqlcipher encryption
- **Search**: fastembed (embedding models)
- **AI**: llama-cpp-rs + TinyLlama 1.1B (Phase 2)
- **GUI**: Tauri (Rust + Web)
- **CLI**: clap for argument parsing

## 🤝 Contributing

Contributions are welcome! This project is built for learning Rust, so beginner-friendly PRs are encouraged.

### Development Setup

```bash
# Clone the repo
git clone https://github.com/yourusername/jot.git
cd jot

# Install dependencies
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- monitor
```

### Areas for Contribution

- 🐛 Bug fixes
- 📝 Documentation improvements
- ✨ New search algorithms
- 🎨 UI/UX enhancements
- 🔧 Performance optimizations
- 🧪 Test coverage

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Embeddings via [fastembed](https://github.com/Anush008/fastembed-rs)
- AI via [llama.cpp](https://github.com/ggerganov/llama.cpp)
- Inspired by the need to remember things better

## 💬 Support

- 📫 Issues: [GitHub Issues](https://github.com/jeffawe/jot/issues)
- 💭 Discussions: [GitHub Discussions](https://github.com/jeffawe/jot/discussions)
- 🐦 Twitter: [@yourhandle](https://twitter.com/awagu_jeffery)

---

**Remember**: Your digital memory, always at your fingertips. Never scroll through history again.

Built with ❤️ and Rust 🦀