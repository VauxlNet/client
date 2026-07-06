# Data Interface Contract

This is the contract between the web UI and the Rust core. It is the keystone of
the client: the UI depends only on this contract, so the shell (Tauri) and the
protocol (Matrix) stay swappable, and the security-critical work stays in Rust.

## Source of truth

- Rust types and the trait live in `core/src/`:
  - `model.rs` (rooms, messages, members, content), `event.rs` (reactive events),
    `error.rs`, `ids.rs`, `backend.rs` (the `ChatBackend` trait).
- `src/bindings.ts` is **generated** from those Rust types by `tauri-specta`.
  Do not edit it by hand. Regenerate with `cargo test -p vauxl-app export_bindings`.

Rust is the single source of truth; the TypeScript cannot drift from it.

## The seam

```
Web UI (untrusted, holds no keys)
    │  commands.*()           ▲  events.coreEventMsg.listen()
    ▼                         │
Tauri bridge  (#[tauri::command] fns + emitted events)
    │
    ▼
Arc<dyn ChatBackend>   ──  MockBackend    (today: canned data, real UI)
                       └─  MatrixBackend  (later: matrix-rust-sdk + vodozemac)
```

The UI calls commands and subscribes to events. It never receives a key, an
access token, or raw HTML.

## Domain model (Discord shaped, Matrix underneath)

- `Space` is a community ("server", a Matrix Space). `Room` is a channel or DM
  (a Matrix room) with a `RoomKind` of `Text | Voice | Announcement | DirectMessage`.
- `Message` carries a `MessageContent` union (`Text`, `Image`, `File`, `Audio`,
  `Redacted`, `Unsupported`), a `SendState` (local echo lifecycle), and a
  `SenderTrust` for the per-message verification indicator.
- IDs (`UserId`, `RoomId`, ...) are opaque strings the UI never parses.

See `core/src/model.rs` for the exact fields.

## Commands (UI to core)

Each is a method on `ChatBackend` and a generated `commands.*` function. They
return `{ status: "ok", data } | { status: "error", error: CoreError }`.

| Area | Commands |
|------|----------|
| Session | `login`, `restoreSession`, `logout` |
| Spaces/rooms | `listSpaces`, `listRooms`, `getMembers` |
| Timeline | `loadTimeline`, `loadOlder`, `sendMessage`, `editMessage`, `redactMessage`, `toggleReaction`, `markRead`, `setTyping` |
| Media | `uploadMedia` |
| E2EE verification | `requestVerification`, `confirmSas`, `cancelVerification` |

## Events (core to UI)

One stream, `events.coreEventMsg`, whose `payload.event` is a `CoreEvent`:
`Session`, `Sync`, `RoomUpserted`, `RoomRemoved`, `SpaceUpserted`, `Timeline`
(`Added` / `Updated` / `Removed`), `Typing`, `Receipt`, `Presence`, `Verification`.

The UI is fully reactive: it calls a command, then reflects the resulting events.
For example `sendMessage` returns the new id, and the appended message arrives as
a `Timeline` `Added` event.

## Security boundary

This is why the core is Rust and the UI is treated as untrusted:

- **No keys in the UI.** Sessions, tokens, and crypto live behind the trait.
- **`SafeHtml`** is constructed only after sanitizing in the core. The UI renders
  it without re-sanitizing; raw HTML never crosses the boundary.
- **`MediaRef`** is an opaque handle. Encrypted media is fetched and decrypted in
  Rust and served to the webview via a custom protocol (for example
  `vauxl-media://{id}`); the UI never holds media keys.
- **Password** crosses once into `login` and is never retained by the UI. OIDC/SSO
  (password never touches our code) is the more secure path to add later.
- **`CoreError::Internal`** carries no detail; internal errors are logged in the
  core, not leaked to the UI.

Follow-ups before this is production grade:

- A strict Content Security Policy is now set for production builds via
  `app.security.csp` in `tauri.conf.json`; `app.security.devCsp` stays `null` so
  Vite HMR works in dev. Revisit the policy (notably `connect-src`) when the
  Matrix backend lands, since the webview will then reach a homeserver.
- Encrypt the local store at rest.
- Metadata caveat: Matrix E2EE protects message content, not metadata. A
  homeserver still sees who is in which rooms and timing. Self-hosting for you and
  your friends keeps that metadata on infrastructure you control.

## Swapping the mock for Matrix

1. Add a `MatrixBackend` in `core` implementing `ChatBackend` over
   `matrix-rust-sdk`. Reuse its `vodozemac` crypto; write none yourself.
2. Change the constructor in `src-tauri/src/lib.rs` `run` from `MockBackend::new()`
   to your `MatrixBackend`. The UI and the bindings are unchanged.
