use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket, CloseEvent, Event};
use yew::prelude::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use gloo::timers::callback::Timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStatusUpdate {
    pub analysis_id: i64,
    pub status: String,
    pub progress: Option<u8>, // 0-100
    pub message: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "analysis_status")]
    AnalysisStatus(AnalysisStatusUpdate),
    #[serde(rename = "system_status")]
    SystemStatus { online: bool, message: String },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

pub struct WebSocketService {
    websocket: Option<WebSocket>,
    on_message: Callback<AnalysisStatusUpdate>,
    on_system_status: Callback<(bool, String)>,
    reconnect_timeout: Option<Timeout>,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
}

impl Clone for WebSocketService {
    fn clone(&self) -> Self {
        Self {
            websocket: None, // Don't clone the websocket connection
            on_message: self.on_message.clone(),
            on_system_status: self.on_system_status.clone(),
            reconnect_timeout: None, // Don't clone timeout
            reconnect_attempts: self.reconnect_attempts,
            max_reconnect_attempts: self.max_reconnect_attempts,
        }
    }
}

impl WebSocketService {
    pub fn new(
        on_message: Callback<AnalysisStatusUpdate>,
        on_system_status: Callback<(bool, String)>,
    ) -> Self {
        Self {
            websocket: None,
            on_message,
            on_system_status,
            reconnect_timeout: None,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
        }
    }

    pub fn connect(&mut self) -> Result<(), JsValue> {
        let protocol = if web_sys::window()
            .unwrap()
            .location()
            .protocol()
            .unwrap()
            .starts_with("https")
        {
            "wss"
        } else {
            "ws"
        };

        let host = web_sys::window()
            .unwrap()
            .location()
            .host()
            .unwrap();

        let ws_url = format!("{}://{}/ws", protocol, host);
        web_sys::console::log_1(&format!("Connecting to WebSocket: {}", ws_url).into());

        let websocket = WebSocket::new(&ws_url)?;
        websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // Clone callbacks for closures
        let on_message = self.on_message.clone();
        let on_system_status = self.on_system_status.clone();

        // Setup message handler
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let message_str = txt.as_string().unwrap_or_default();
                
                if let Ok(ws_message) = serde_json::from_str::<WebSocketMessage>(&message_str) {
                    match ws_message {
                        WebSocketMessage::AnalysisStatus(status) => {
                            on_message.emit(status);
                        }
                        WebSocketMessage::SystemStatus { online, message } => {
                            on_system_status.emit((online, message));
                        }
                        WebSocketMessage::Ping => {
                            // Respond with pong (handled by browser automatically for standard pings)
                            web_sys::console::log_1(&"Received ping".into());
                        }
                        WebSocketMessage::Pong => {
                            web_sys::console::log_1(&"Received pong".into());
                        }
                    }
                } else {
                    web_sys::console::error_1(&format!("Failed to parse WebSocket message: {}", message_str).into());
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // Setup connection opened handler
        let onopen_callback = Closure::wrap(Box::new(move |_: Event| {
            web_sys::console::log_1(&"WebSocket connection opened".into());
        }) as Box<dyn FnMut(Event)>);

        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // Setup error handler
        let onerror_callback = Closure::wrap(Box::new(move |e: Event| {
            web_sys::console::error_1(&"WebSocket error occurred".into());
            web_sys::console::error_1(&e);
        }) as Box<dyn FnMut(Event)>);

        websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // Setup close handler with reconnection logic
        let ws_clone = Rc::new(std::cell::RefCell::new(None::<WebSocket>));
        let _ws_for_close = ws_clone.clone();

        let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
            web_sys::console::log_1(&format!("WebSocket connection closed: {}", e.reason()).into());
            
            // TODO: Implement reconnection logic here if needed
            // This would require more complex state management
        }) as Box<dyn FnMut(CloseEvent)>);

        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        self.websocket = Some(websocket);
        self.reconnect_attempts = 0;

        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Some(ws) = &self.websocket {
            let _ = ws.close();
        }
        self.websocket = None;

        if let Some(timeout) = self.reconnect_timeout.take() {
            timeout.cancel();
        }
    }

    pub fn subscribe_to_analysis(&self, analysis_id: i64) -> Result<(), JsValue> {
        if let Some(ws) = &self.websocket {
            let message = serde_json::json!({
                "type": "subscribe",
                "analysis_id": analysis_id
            });

            ws.send_with_str(&message.to_string())
        } else {
            Err(JsValue::from_str("WebSocket not connected"))
        }
    }

    pub fn unsubscribe_from_analysis(&self, analysis_id: i64) -> Result<(), JsValue> {
        if let Some(ws) = &self.websocket {
            let message = serde_json::json!({
                "type": "unsubscribe",
                "analysis_id": analysis_id
            });

            ws.send_with_str(&message.to_string())
        } else {
            Err(JsValue::from_str("WebSocket not connected"))
        }
    }

    pub fn is_connected(&self) -> bool {
        self.websocket
            .as_ref()
            .map(|ws| ws.ready_state() == WebSocket::OPEN)
            .unwrap_or(false)
    }
}

impl Drop for WebSocketService {
    fn drop(&mut self) {
        self.disconnect();
    }
}

// Hook for using WebSocket in Yew components
#[hook]
pub fn use_websocket(
    on_message: Callback<AnalysisStatusUpdate>,
    on_system_status: Callback<(bool, String)>,
) -> UseStateHandle<Option<WebSocketService>> {
    let websocket_service = use_state(|| None::<WebSocketService>);

    // Connect on mount
    {
        let websocket_service = websocket_service.clone();
        let on_message = on_message.clone();
        let on_system_status = on_system_status.clone();

        use_effect_with((), move |_| {
            let mut service = WebSocketService::new(on_message, on_system_status);
            
            if let Err(e) = service.connect() {
                web_sys::console::error_1(&format!("Failed to connect WebSocket: {:?}", e).into());
            } else {
                websocket_service.set(Some(service));
            }

            // Cleanup on unmount
            let websocket_service = websocket_service.clone();
            move || {
                if let Some(mut service) = (*websocket_service).clone() {
                    service.disconnect();
                }
            }
        });
    }

    websocket_service
}
