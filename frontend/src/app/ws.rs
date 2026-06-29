use crate::app::App;
use crate::app::Msg;
use crate::types::SystemStats;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};
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

        let ws = WebSocket::new(&ws_url);
        let ws = match ws {
            Ok(w) => w,
            Err(e) => {
                ctx.link().send_message(Msg::WsError(format!("WS open error: {:?}", e)));
                return;
            }
        };

        // OnMessage callback
        let link = ctx.link().clone();
        let onmessage_callback = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            if let Some(txt) = e.data().as_string() {
                if let Ok(stats) = serde_json::from_str::<SystemStats>(&txt) {
                    link.send_message(Msg::UpdateStats(stats));
                }
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // OnError callback
        let link = ctx.link().clone();
        let onerror_callback = Closure::<dyn FnMut(JsValue)>::new(move |e: JsValue| {
            link.send_message(Msg::WsError(format!("Connection error: {:?}", e)));
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        self.ws = Some(ws);
    }
}
