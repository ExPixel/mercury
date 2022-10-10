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

    'select_loop: loop {
        tokio::select! {
            maybe_msg = socket.recv() => {
                if let Some(msg) = maybe_msg {
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
            }
        }
    }

    debug!("websocket disconnecting");
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
    }
}

async fn on_recv_ws_message(socket: &mut WebSocket, msg: Message, state: &mut SocketState) {
    trace!(msg = debug(&msg), "received websocket message");

    let msg: WsMessageFromClient = if let Message::Text(msg) = msg {
        match serde_json::from_str(&msg) {
            Ok(msg) => msg,
            Err(error) => {
                error!(error = debug(error), "deserialization error");
                return;
            }
        }
    } else {
        error!(message = debug(msg), "invalid message type");
        return;
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
