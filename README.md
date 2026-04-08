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
| `Ctrl+N` | Create a new channel |
| `Ctrl+L` | Browse all channels |
| `Ctrl+M` | Show members of the active channel |
| `Ctrl+U` | Add a user to the active channel (owner only, private channels) |
| `Ctrl+K` | Remove a user from the active channel (owner only) |
| `Ctrl+J` | Join the active channel (public channels) |
| `Ctrl+O` | Logout |
| `Ctrl+C` | Quit |

All `Ctrl+*` actions open a centered modal popup. Inside a modal: `Tab` switches fields, `Space` toggles checkboxes, `Enter` confirms, `Esc` cancels.

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
- Your login token is saved to `~/.config/termchat/token` so you stay logged in (use `Ctrl+O` to logout)

## Channels

All channel and membership management lives in the TUI — no `curl` required.

### Create a channel

Press `Ctrl+N` to open the **Create Channel** popup. Fill in:

- **Name** — channel name (required, must be unique)
- **Description** — optional one-liner
- **Private** — `Tab` to the privacy field, then `Space` to toggle. Private channels can only be joined by being added by the owner.

`Enter` creates the channel. You become the owner and are joined automatically.

### Browse channels

Press `Ctrl+L` to open the **Channels** popup. Use `↑` / `↓` to move and `Enter` to switch the chat view to that channel. Private channels are tagged `[priv]`.

### List channel members

Select a channel in the sidebar, then press `Ctrl+M` to see everyone in it.

### Join a public channel

Select the channel (via `Ctrl+L` or the sidebar), then press `Ctrl+J`. Private channels reject self-join — ask the owner to add you.

### Add a user to a channel (owner only)

Open the channel, press `Ctrl+U`, type the username, `Enter`. Required for getting people into private channels.

### Remove a user from a channel (owner only)

Open the channel, press `Ctrl+K`, highlight the user with `↑` / `↓`, then `Enter`. The channel owner cannot be removed.

### Logout

Press `Ctrl+O`, then `y` to confirm. This deletes the cached token at `~/.config/termchat/token` and returns you to the login screen.

For the full server endpoint reference (DMs, WebSocket protocol, etc.) see the [termchat-server README](https://github.com/enesdindas/termchat-server#api).

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
