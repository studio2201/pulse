#![allow(clippy::collapsible_if, clippy::unnecessary_map_or)]

mod app;
mod storage;
mod types;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
