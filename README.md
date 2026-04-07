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

**First launch:** press `Ctrl+R`, type a username and password (use `Tab` to switch fields), then `Enter` to register. You'll be signed in immediately.

**Later launches:** your JWT is cached at `~/.config/termchat/token`, so termchat auto-logs you in. Delete that file to force the login screen back, or to sign in as a different user.

**Logging in on another machine:** run `termchat`, type your username, `Tab`, password, `Enter`.

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

## Channels

Channel creation and membership are managed over the server's REST API. The TUI shows channels you already belong to, but it does not yet have in-app commands for creating or joining them — so the steps below use `curl`.

All examples assume `TERMCHAT_SERVER` is set (defaults to `https://termchat-server-09qq.onrender.com`):

```bash
export TERMCHAT_SERVER=https://termchat-server-09qq.onrender.com
```

### Get an auth token

```bash
TOKEN=$(curl -s -X POST "$TERMCHAT_SERVER/auth/login" \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"hunter2"}' | jq -r .token)
```

If you're already logged in through the TUI, you can reuse the cached token instead:

```bash
TOKEN=$(cat ~/.config/termchat/token)
```

### Create a channel

The creator is automatically recorded as the owner and joined to the channel.

```bash
curl -X POST "$TERMCHAT_SERVER/api/channels" \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"general","description":"Company-wide chat"}'
```

The response contains the new channel's `id` — you'll need it for the commands below.

### List channels and members

```bash
# All channels on the server
curl -s "$TERMCHAT_SERVER/api/channels" -H "Authorization: Bearer $TOKEN"

# Members of channel 42
curl -s "$TERMCHAT_SERVER/api/channels/42/members" -H "Authorization: Bearer $TOKEN"
```

### Add a user to a channel

There is **no admin invite endpoint** — a user joins a channel themselves using their own token. To "add" someone, share the channel `id` and have them run:

```bash
curl -X POST "$TERMCHAT_SERVER/api/channels/42/join" \
  -H "Authorization: Bearer $TOKEN"
```

### Remove a user from a channel

Symmetrically, users remove themselves. There is no admin-side "kick" today.

```bash
curl -X POST "$TERMCHAT_SERVER/api/channels/42/leave" \
  -H "Authorization: Bearer $TOKEN"
```

### Seeing changes in the TUI

The client loads your channel list at login, so after creating or joining a channel you need to **quit and relaunch `termchat`** for it to appear in the sidebar.

For the full endpoint reference (DMs, WebSocket protocol, etc.) see the [termchat-server README](https://github.com/enesdindas/termchat-server#api).

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
