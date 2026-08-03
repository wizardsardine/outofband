mod app;
mod components;
mod hooks;
mod queue;
mod slipstream;
mod tokens;
mod unpack;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
