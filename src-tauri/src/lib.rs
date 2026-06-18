//! Tauri shell for the Vauxl client.
//!
//! This layer is thin on purpose: it owns the window, exposes the `ChatBackend`
//! contract to the webview as typed commands and events, and forwards core
//! events to the UI. All domain and (later) security-critical logic lives in
//! `vauxl-core`. Today the backend is `MockBackend`; swapping in a
//! matrix-rust-sdk backend changes only the constructor in `run`.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder, Event};

use vauxl_core::{
    ChatBackend, CoreError, CoreEvent, LoginRequest, MediaRef, Member, MessageId, MockBackend,
    OutgoingContent, Room, RoomId, SessionInfo, Space, TimelineChunk, UserId,
};

/// Held in Tauri-managed state. Commands clone the `Arc` out before awaiting so
/// the state guard is never held across an await point.
struct AppState {
    backend: Arc<dyn ChatBackend>,
}

/// Wrapper so the core's `CoreEvent` can be a typed tauri-specta event without
/// the core depending on Tauri. The frontend reads `payload.event`.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct CoreEventMsg {
    pub event: CoreEvent,
}

// ---------- Commands (thin wrappers over the ChatBackend trait) ----------

#[tauri::command]
#[specta::specta]
async fn login(
    state: tauri::State<'_, AppState>,
    req: LoginRequest,
) -> Result<SessionInfo, CoreError> {
    let backend = state.backend.clone();
    backend.login(req).await
}

#[tauri::command]
#[specta::specta]
async fn restore_session(
    state: tauri::State<'_, AppState>,
) -> Result<Option<SessionInfo>, CoreError> {
    let backend = state.backend.clone();
    backend.restore_session().await
}

#[tauri::command]
#[specta::specta]
async fn logout(state: tauri::State<'_, AppState>) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.logout().await
}

#[tauri::command]
#[specta::specta]
async fn list_spaces(state: tauri::State<'_, AppState>) -> Result<Vec<Space>, CoreError> {
    let backend = state.backend.clone();
    backend.list_spaces().await
}

#[tauri::command]
#[specta::specta]
async fn list_rooms(state: tauri::State<'_, AppState>) -> Result<Vec<Room>, CoreError> {
    let backend = state.backend.clone();
    backend.list_rooms().await
}

#[tauri::command]
#[specta::specta]
async fn get_members(
    state: tauri::State<'_, AppState>,
    room: RoomId,
) -> Result<Vec<Member>, CoreError> {
    let backend = state.backend.clone();
    backend.get_members(room).await
}

#[tauri::command]
#[specta::specta]
async fn load_timeline(
    state: tauri::State<'_, AppState>,
    room: RoomId,
    limit: u32,
) -> Result<TimelineChunk, CoreError> {
    let backend = state.backend.clone();
    backend.load_timeline(room, limit).await
}

#[tauri::command]
#[specta::specta]
async fn load_older(
    state: tauri::State<'_, AppState>,
    room: RoomId,
    before: MessageId,
    limit: u32,
) -> Result<TimelineChunk, CoreError> {
    let backend = state.backend.clone();
    backend.load_older(room, before, limit).await
}

#[tauri::command]
#[specta::specta]
async fn send_message(
    state: tauri::State<'_, AppState>,
    room: RoomId,
    content: OutgoingContent,
) -> Result<MessageId, CoreError> {
    let backend = state.backend.clone();
    backend.send_message(room, content).await
}

#[tauri::command]
#[specta::specta]
async fn edit_message(
    state: tauri::State<'_, AppState>,
    room: RoomId,
    target: MessageId,
    content: OutgoingContent,
) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.edit_message(room, target, content).await
}

#[tauri::command]
#[specta::specta]
async fn redact_message(
    state: tauri::State<'_, AppState>,
    room: RoomId,
    target: MessageId,
) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.redact_message(room, target).await
}

#[tauri::command]
#[specta::specta]
async fn toggle_reaction(
    state: tauri::State<'_, AppState>,
    room: RoomId,
    target: MessageId,
    key: String,
) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.toggle_reaction(room, target, key).await
}

#[tauri::command]
#[specta::specta]
async fn mark_read(
    state: tauri::State<'_, AppState>,
    room: RoomId,
    up_to: MessageId,
) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.mark_read(room, up_to).await
}

#[tauri::command]
#[specta::specta]
async fn set_typing(
    state: tauri::State<'_, AppState>,
    room: RoomId,
    typing: bool,
) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.set_typing(room, typing).await
}

#[tauri::command]
#[specta::specta]
async fn upload_media(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<MediaRef, CoreError> {
    let backend = state.backend.clone();
    backend.upload_media(path).await
}

#[tauri::command]
#[specta::specta]
async fn request_verification(
    state: tauri::State<'_, AppState>,
    user: UserId,
) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.request_verification(user).await
}

#[tauri::command]
#[specta::specta]
async fn confirm_sas(state: tauri::State<'_, AppState>, flow: String) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.confirm_sas(flow).await
}

#[tauri::command]
#[specta::specta]
async fn cancel_verification(
    state: tauri::State<'_, AppState>,
    flow: String,
) -> Result<(), CoreError> {
    let backend = state.backend.clone();
    backend.cancel_verification(flow).await
}

// ---------- Builder, bindings export, app entry ----------

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            login,
            restore_session,
            logout,
            list_spaces,
            list_rooms,
            get_members,
            load_timeline,
            load_older,
            send_message,
            edit_message,
            redact_message,
            toggle_reaction,
            mark_read,
            set_typing,
            upload_media,
            request_verification,
            confirm_sas,
            cancel_verification,
        ])
        .events(collect_events![CoreEventMsg])
}

fn ts() -> specta_typescript::Typescript {
    specta_typescript::Typescript::default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(ts(), "../src/bindings.ts")
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let mock = Arc::new(MockBackend::new());

            // Forward core events to the webview.
            let mut rx = mock.subscribe();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            let _ = CoreEventMsg { event }.emit(&handle);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });

            // Fake live traffic so the prototype feels alive.
            tauri::async_runtime::spawn(mock.clone().run_demo_traffic());

            let backend: Arc<dyn ChatBackend> = mock;
            app.manage(AppState { backend });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_bindings() {
        specta_builder()
            .export(ts(), "../src/bindings.ts")
            .expect("failed to export bindings");
    }
}
