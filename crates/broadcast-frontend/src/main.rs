mod app;
mod components;
mod hooks;
mod queue;
mod tokens;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
