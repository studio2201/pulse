use crate::app::App;
use crate::app::Msg;
use crate::types::SystemStats;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{MessageEvent, WebSocket, CloseEvent};
use yew::prelude::*;

impl App {
    pub fn connect_ws(&mut self, ctx: &Context<Self>) {
        if self.ws.is_some() {
            return;
        }

        let window = web_sys::window().expect("no global window exists");
        let location = window.location();
        let host = location.host().expect("failed to get host");
        let protocol = location.protocol().expect("failed to get protocol");

        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        let ws_url = format!("{}//{}/api/stats/ws", ws_protocol, host);

        web_sys::console::log_1(&JsValue::from_str(&format!("[WS] Connecting to {}", ws_url)));
        self.terminal_logs.push(format!("[WS] Connecting to {}...", ws_url));

        let ws = WebSocket::new(&ws_url);
        let ws = match ws {
            Ok(w) => w,
            Err(e) => {
                let err_msg = format!("WS constructor error: {:?}", e);
                web_sys::console::error_1(&JsValue::from_str(&err_msg));
                ctx.link().send_message(Msg::WsError(err_msg));
                return;
            }
        };

        // OnOpen callback
        let link = ctx.link().clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            web_sys::console::log_1(&JsValue::from_str("[WS] Connection opened successfully."));
            link.send_message(Msg::WsError("[WS] Connection established. Dashboard online.".to_string()));
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // OnMessage callback
        let link = ctx.link().clone();
        let onmessage_callback = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            let data = e.data();
            web_sys::console::log_2(&JsValue::from_str("[WS] Received raw data:"), &data);
            
            if let Some(txt) = data.as_string() {
                match serde_json::from_str::<SystemStats>(&txt) {
                    Ok(stats) => {
                        link.send_message(Msg::UpdateStats(stats));
                    }
                    Err(err) => {
                        let err_msg = format!("[WS] JSON Parse error: {:?} (Data: {})", err, txt);
                        web_sys::console::error_1(&JsValue::from_str(&err_msg));
                        link.send_message(Msg::WsError(err_msg));
                    }
                }
            } else {
                let err_msg = format!("[WS] Received non-string data (type: {})", data.js_typeof().as_string().unwrap_or_default());
                web_sys::console::warn_1(&JsValue::from_str(&err_msg));
                link.send_message(Msg::WsError(err_msg));
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // OnClose callback
        let link = ctx.link().clone();
        let onclose_callback = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
            let close_msg = format!("[WS] Closed (code: {}, clean: {})", e.code(), e.was_clean());
            web_sys::console::warn_1(&JsValue::from_str(&close_msg));
            link.send_message(Msg::WsError(close_msg));
        });
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        // OnError callback
        let link = ctx.link().clone();
        let onerror_callback = Closure::<dyn FnMut(JsValue)>::new(move |e: JsValue| {
            let err_msg = format!("[WS] Socket error: {:?}", e);
            web_sys::console::error_1(&JsValue::from_str(&err_msg));
            link.send_message(Msg::WsError(err_msg));
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        self.ws = Some(ws);
    }
}
