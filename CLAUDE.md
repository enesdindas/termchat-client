# termchat-client — Claude Context

## Build & Run

```bash
source "$HOME/.cargo/env"   # if cargo not on PATH
cargo build                 # dev build
cargo build --release       # release build → target/release/termchat
cargo run                   # run directly
cargo test                  # run tests
make run / make test        # Makefile aliases
```

## Architecture

```
src/main.rs              Entry: terminal setup (raw mode, alternate screen), App::run(), panic hook
src/app.rs               App struct — central tokio::select! event loop
src/config.rs            Config::load() from env; token persistence to ~/.config/termchat/token
src/api/rest.rs          RestClient — reqwest wrapper for all HTTP endpoints
src/api/ws.rs            WsConnection — tokio-tungstenite; spawns reader/writer tasks, exposes mpsc channels
src/models/              Serde structs mirroring server JSON exactly
src/state/app_state.rs   AppState — all mutable UI state; no I/O
src/ui/                  Pure rendering functions using ratatui — no mutation
src/events/handler.rs    Key dispatch → AppState mutations or WS sends
src/tests/               Unit tests for models (serde) and app_state (state mutations)
```

## Key Design Decisions

- **Event loop**: `tokio::select!` in `App::run` multiplexes three sources: crossterm `EventStream`, WebSocket `mpsc::Receiver<WsEnvelope>`, and a 100ms tick for processing deferred auth actions.
- **Deferred auth**: Login/register set `pending_login`/`pending_register` flags on the tick, which are processed asynchronously. This avoids blocking the render loop on HTTP calls.
- **No terminal blocking on I/O**: WsConnection spawns two independent tokio tasks (reader + writer). All WS I/O is fully decoupled from the render path.
- **State is pure**: `AppState` has no async methods — it only holds data. I/O lives in `app.rs` and `api/`.
- **Token persistence**: Saved to `~/.config/termchat/token`; loaded on startup to auto-login.
- **Unread tracking**: `add_channel_message` / `add_dm_message` increment unread counts when the message's conversation isn't currently selected; `select_channel` / `select_dm` clear them.

## WebSocket Protocol

Envelope: `{"type":"<event>","payload":{...}}`

Client sends: `subscribe`, `message.send`, `dm.send`, `typing.start`, `typing.stop`
Server sends: `message.new`, `dm.new`, `typing.indicator`, `error`

Connect: `ws://localhost:8080/ws?token=<JWT>`

## Testing

Tests live in `src/tests/`. They are pure unit tests with no I/O — `AppState` and model serde only.
Add new state tests in `app_state_test.rs` and new model tests in `models_test.rs`.

## Environment Variables

| Var | Default |
|---|---|
| `TERMCHAT_SERVER` | `http://localhost:8080` |
