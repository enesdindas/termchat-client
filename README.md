# termchat-client

Rust terminal UI client for termchat. Built with [ratatui](https://ratatui.rs/) and [tokio](https://tokio.rs/).

## Requirements

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- A running `termchat-server` instance

## Quick Start

```bash
# Run with default server (localhost:8080)
cargo run

# Or point to a different server
TERMCHAT_SERVER=http://myserver:8080 cargo run
```

## Usage

### Login Screen
| Key | Action |
|-----|--------|
| `Tab` | Switch between username / password fields |
| `Enter` | Login |
| `Ctrl+R` | Register new account |
| `Ctrl+C` | Quit |

### Main Screen
| Key | Action |
|-----|--------|
| Type | Compose message |
| `Enter` | Send message |
| `Alt+↑` / `Alt+↓` | Navigate channels/DMs in sidebar |
| `PageUp` / `PageDown` | Scroll chat history |
| `Ctrl+C` | Quit |

### Layout

```
┌──────────────────┬───────────────────────────────────────────────┐
│ Channels & DMs   │ # general                                      │
│ ─────────────── │                                               │
│ # general  ●    │ [12:00] alice: Hello everyone!                │
│ # random        │ [12:01] bob: Hey there                        │
│ ─────────────── │                                               │
│ @ bob  (2)      │                                               │
│ @ charlie       │                                               │
│                  │                                               │
│                  ├───────────────────────────────────────────────┤
│                  │ Message (Enter to send) █                     │
└──────────────────┴───────────────────────────────────────────────┘
```

- `#` items are channels
- `@` items are direct messages
- Numbers in parentheses show unread message count

## Configuration

| Variable | Default | Description |
|---|---|---|
| `TERMCHAT_SERVER` | `http://localhost:8080` | Server base URL |

Your JWT token is automatically saved to `~/.config/termchat/token` after login so you stay logged in between sessions.

## Development

```bash
make test    # Run all tests
make lint    # Run clippy
make build   # Release build → target/release/termchat
make clean   # Clean build artifacts
```
