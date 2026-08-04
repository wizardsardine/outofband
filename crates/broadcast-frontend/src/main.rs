mod app;
mod components;
mod guides;
mod hooks;
mod i18n;
mod queue;
mod slipstream;
mod tokens;
mod unpack;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
