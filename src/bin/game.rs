const APP_NAME: &str = "support_game";

use known_folders::{get_known_folder_path, KnownFolder};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

fn window_conf() -> Conf {
    Conf {
        window_title: "Window name".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct State {}

impl State {
    fn save(&mut self) {
        let folder = get_known_folder_path(KnownFolder::LocalAppData)
            .unwrap()
            .join(APP_NAME);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("data.rsn"), rsn::to_string(self)).unwrap();
    }
}

#[macroquad::main(window_conf)]
async fn main() -> anyhow::Result<()> {
    let data = known_folders::get_known_folder_path(known_folders::KnownFolder::LocalAppData)
        .unwrap()
        .join(APP_NAME)
        .join("data.rsn");

    let state = if data.exists() {
        match std::fs::read_to_string(data) {
            Ok(data) => Some(rsn::from_str::<State>(&data).unwrap()),
            _ => None,
        }
    } else {
        None
    };

    loop {
        clear_background(RED);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);

        draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
