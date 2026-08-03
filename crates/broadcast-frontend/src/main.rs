mod app;
mod components;
mod hooks;
mod tokens;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
