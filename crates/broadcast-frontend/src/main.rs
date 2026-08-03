mod app;
mod components;
mod hooks;
mod i18n;
mod queue;
mod slipstream;
mod tokens;
mod unpack;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
