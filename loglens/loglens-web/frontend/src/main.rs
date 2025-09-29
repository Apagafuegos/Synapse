mod app;
mod components;
mod pages;
mod router;
mod services;
mod types;

pub use router::Route;
use app::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}