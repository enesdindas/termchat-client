# termchat

A terminal chat application with channels and direct messages. Runs entirely in your terminal.

## Install

**macOS / Linux via curl (recommended):**

```bash
curl -fsSL https://raw.githubusercontent.com/enesdindas/termchat-client/main/install.sh | sh
```

**macOS via Homebrew:**

```bash
brew tap enesdindas/termchat
brew install termchat
```

## Run

```bash
termchat
```

That's it. The app connects to the public server automatically. Register a new account on first launch.

## Usage

### Login screen

| Key | Action |
|-----|--------|
| `Tab` | Switch between username / password fields |
| `Enter` | Login |
| `Ctrl+R` | Register new account |
| `Ctrl+C` | Quit |

### Main screen

| Key | Action |
|-----|--------|
| Type | Compose message |
| `Enter` | Send message |
| `↑` / `↓` | Navigate channels / DMs in sidebar (when input is empty) |
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

- `#` items are channels — join public rooms and chat with everyone in them
- `@` items are direct messages — private 1:1 conversations
- Numbers in parentheses show unread message count
- Your login token is saved to `~/.config/termchat/token` so you stay logged in

## Connect to a different server

By default termchat connects to the hosted server at `https://termchat-server-09qq.onrender.com`. To use your own:

```bash
TERMCHAT_SERVER=http://localhost:8080 termchat
```

## Build from source

Requires Rust 1.70+ ([install via rustup](https://rustup.rs/)).

```bash
git clone https://github.com/enesdindas/termchat-client
cd termchat-client
cargo build --release
./target/release/termchat
```

## Server

The backend is open source at [enesdindas/termchat-server](https://github.com/enesdindas/termchat-server). Self-hosting instructions are in that repo's README.
