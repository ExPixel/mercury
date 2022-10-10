use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
    Extension,
};
use serde::{Deserialize, Serialize};
use storage::{Storage, StorageEvent};
use tracing::{debug, error, trace};

pub async fn listen(ws: WebSocketUpgrade, storage: Extension<Storage>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, storage.0))
}

async fn handle_socket(mut socket: WebSocket, storage: Storage) {
    debug!("websocket connected");

    let mut event_rx = storage.subscribe();
    let mut state = SocketState::default();
    let mut heartbeat = tokio::time::interval(Duration::from_secs(30));

    'select_loop: loop {
        tokio::select! {
            maybe_msg = socket.recv() => {
                if let Some(msg) = maybe_msg {
                    state.active = true;
                    match msg {
                        Ok(msg) => on_recv_ws_message(&mut socket, msg, &mut state).await,
                        Err(err) => {
                            error!(error = debug(err), "websocket error");
                            break 'select_loop;
                        }
                    }
                } else {
                    break 'select_loop;
                }
            },

            store_event = event_rx.recv() => {
                if let Ok(event) = store_event {
                    on_recv_storage_event(&mut socket, event, &mut state).await;
                } else {
                    error!("storage event recv error");
                    break 'select_loop;
                }
            },

            _ = heartbeat.tick() => {
                trace!("sending heartbeat ping");
                if let Err(err) = socket.send(Message::Ping(vec![0xEF, 0xBE, 0xAD, 0xDE])).await {
                    error!(error = debug(err), "socket send error");
                    break 'select_loop;
                }
            },
        }

        if state.closed {
            debug!("socket state closed is true, exiting loop");
            break 'select_loop;
        }

        if state.active {
            state.active = false;
            heartbeat.reset();
        }
    }

    debug!("exited websocket loop");
}

async fn on_recv_storage_event(
    socket: &mut WebSocket,
    event: StorageEvent,
    state: &mut SocketState,
) {
    debug!(event = debug(event), "received storage event");

    if state.listen_for_new_mail {
        let msg = serde_json::to_string(&WsMessageFromServer::NewMailAvailable)
            .expect("serialization error");

        if let Err(error) = socket.send(Message::Text(msg)).await {
            error!(error = debug(error), "socket send error");
        }
        state.active = true;
    }
}

async fn on_recv_ws_message(_socket: &mut WebSocket, msg: Message, state: &mut SocketState) {
    trace!(msg = debug(&msg), "received websocket message");

    let msg: WsMessageFromClient = match msg {
        Message::Text(msg) => match serde_json::from_str(&msg) {
            Ok(msg) => msg,
            Err(error) => {
                error!(error = debug(error), "deserialization error");
                return;
            }
        },

        Message::Pong(_) => {
            trace!("received pong response");
            return;
        }

        Message::Close(frame) => {
            trace!(frame = debug(frame), "close message received");
            state.closed = true;
            return;
        }

        _ => {
            error!(message = debug(msg), "invalid message type");
            return;
        }
    };

    match msg {
        WsMessageFromClient::ListenForNewMail => {
            debug!("client is now listening for new mail");
            state.listen_for_new_mail = true;
        }
        WsMessageFromClient::Heartbeat => trace!("received heartbeat message"),
    }
}

#[derive(Default)]
pub struct SocketState {
    listen_for_new_mail: bool,
    active: bool,
    closed: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum WsMessageFromClient {
    ListenForNewMail,
    Heartbeat,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum WsMessageFromServer {
    NewMailAvailable,
}
