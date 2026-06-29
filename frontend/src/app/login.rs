use crate::app::App;
use crate::app::Msg;
use yew::prelude::*;

impl App {
    pub fn view_login(&self, ctx: &Context<Self>) -> Html {
        let pin_len = self.pin_length;
        let is_locked = self.lockout_minutes.is_some();
        
        let error_html = self.error_message.as_ref().map(|err| {
            html! { <p id="pin-error" class="pin-error" style="display: block;">{err}</p> }
        });

        html! {
            <div class="login-container">
                <div class="login-box">
                    <div class="login-header">
                        <h2>
                            {
                                if is_locked {
                                    "TOO MANY ATTEMPTS"
                                } else {
                                    "ENTER PIN"
                                }
                            }
                        </h2>
                    </div>
                    <form id="pin-form" onsubmit={ctx.link().callback(|e: SubmitEvent| { e.prevent_default(); Msg::SubmitPin })}>
                        <div class="pin-wrapper">
                            <input
                                type="password"
                                class="pin-input-field"
                                value={self.pin_input.clone()}
                                oninput={ctx.link().callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::PinInputChanged(input.value())
                                })}
                                disabled={is_locked}
                                placeholder={"• ".repeat(pin_len).trim().to_string()}
                                maxlength={pin_len.to_string()}
                                autofocus=true
                            />
                        </div>
                    </form>
                    <div class="pin-status">
                        {
                            if let Some(mins) = self.lockout_minutes {
                                html! {
                                    <p id="lockoutNotice" class="lockout-notice" style="display: block;">
                                        { format!("Too many attempts. Locked out for {} minute(s).", mins) }
                                    </p>
                                }
                            } else {
                                html! {}
                            }
                        }
                        {
                            if let Some(attempts) = self.attempts_left {
                                if attempts < 5 && !is_locked {
                                    html! {
                                        <p id="attemptsRemaining" class="attempts-remaining" style="display: block;">
                                            { format!("{} attempts remaining", attempts) }
                                        </p>
                                    }
                                } else {
                                    html! {}
                                }
                            } else {
                                html! {}
                            }
                        }
                        {error_html}
                    </div>
                </div>
            </div>
        }
    }
}
