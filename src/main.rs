use bevy::prelude::*;
use armilia::AppPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}
