# jot

> Your digital memory. Search everything you've copied or typed, instantly.

> ⚠️ **WARNING: Active Development**  
> jot is currently in active development and not ready for production use. Features are incomplete and bugs are expected.

**jot** is a fully local, privacy-first AI assistant that remembers your clipboard history and terminal commands. Ask questions in natural language and get instant answers, no scrolling, no searching, just results.

[![Release](https://img.shields.io/github/v/release/Jeffawe/Jot)](https://github.com/Jeffawe/Jot/releases)
[![License](https://img.shields.io/github/license/Jeffawe/Jot)](LICENSE)

## ✨ Features

- 🔍 **Natural Language Search** - Find things by meaning, not just keywords
- 🔒 **100% Private** - Everything stays on your machine, encrypted
- ⚡ **Lightning Fast** - Search 10,000+ items in milliseconds
- 🎨 **Dual Interface** - Beautiful GUI or blazing-fast CLI
- 🧠 **Context Aware** - Understands what you're looking for
- 🧩 **Plugin Ready** - Extend Jot with custom Rhai plugins that tap into lifecycle events.

## 🎯 Use Cases

```bash
# Instead of scrolling through terminal history
jotx ask "ssh command for staging server"
→ ssh user@staging.example.com -i ~/.ssh/key.pem

# Find that email you copied hours ago
ja "email address from this morning"
→ john.doe@example.com

# Find that yarn command to run the server
js "yarn"
→ yarn start
```

## 🚀 Quick Start

### macOS & Linux

Copy and paste this into your terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/Jeffawe/Jot/main/install.sh | bash
```

That's it! The installer will:
- ✅ Download the right binary for your system
- ✅ Install Ollama (local AI)
- ✅ Set up shell hooks
- ✅ Start the daemon

**Alternative with wget:**
```bash
wget -qO- https://raw.githubusercontent.com/Jeffawe/Jot/main/install.sh | bash
```

### First Run

```bash
# Start monitoring (runs in background)
jotx run

# Search your history
jotx search "ssh"

# Ask questions
jotx ask "what was that git command from yesterday?"
```

## 📖 How It Works

### Natural Language Search

1. **Monitors** your clipboard and terminal silently in the background
2. **Stores** everything locally in a local SQLite database
3. **Indexes** content using embedding models for semantic search
4. **Searches** using natural language to find what you need

### CLI Commands

```bash
# Start clipboard & terminal monitoring
jotx run

# Search with natural language
jotx ask "docker command with port mapping"
jotx search "yarn" 
```

### GUI Mode

```bash
# Launch the desktop app
jotx gui

# Or use global hotkey (configurable)
Cmd+Shift+J  # macOS default
```

## ⚙️ Configuration

Configuration file: `~/.jotx/config.toml`

```toml
[llm]
provider = "ollama"
api_base = "http://localhost:11434"
model = "qwen2.5:1.5b"
max_tokens = 500
temperature = 0.7
max_history_results = 10

[search]
similarity_threshold = 0.5
max_results = 10
fuzzy_matching = true

[storage]
maintenance_interval_days = 7
```

## 🔒 Privacy & Security

**jot** is built privacy-first:

- ✅ **100% Local** - No data ever leaves your machine
- ✅ **No Telemetry** - Zero analytics or tracking
- ✅ **Configurable Exclusions** - Block apps, files, or patterns (In development)
- ✅ **Open Source** - Fully auditable code
- ✅ **Clean Data** - Clean data stored anytime easily

<!-- ### Exclusion System

Protect sensitive data automatically:

```bash
# Exclude specific apps
jotx config exclude-app "1Password"

# Exclude file patterns
jotx config exclude-pattern "*.env"

# Exclude folders
jotx config exclude-folder "~/Documents/Private"

# View exclusions
jotx config list-exclusions
``` -->

## 🛠️ Tech Stack

- **Language**: Rust 🦀
- **Storage**: SQLite
- **Search**: fastembed (embedding models)
- **AI**: Ollama
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
make dev-build OR cargo build

# Run tests
cargo test
```

### Areas for Contribution

- 🐛 Bug fixes
- 📝 Documentation improvements
- ✨ New search algorithms
- 🎨 UI/UX enhancements
- 🔧 Performance optimizations
- 🧪 Test coverage

## 📄 License

Apache License - see [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Embeddings via [fastembed](https://github.com/Anush008/fastembed-rs)
- AI via [Ollama](https://ollama.com) for running local LLMs
- Inspired by the need to remember things better

## 💬 Support

- 📫 Issues: [GitHub Issues](https://github.com/jeffawe/jot/issues)
- 💭 Discussions: [GitHub Discussions](https://github.com/jeffawe/jot/discussions)
- 🐦 Twitter: [@awagu_jeffery](https://twitter.com/awagu_jeffery)

---

**Remember**: Your digital memory, always at your fingertips. Never scroll through history again.

Built with ❤️ and Rust 🦀