use dino_game::{config::window_conf, game};

#[macroquad::main(window_conf)]
async fn main() {
    game::run().await;
}
