mod player;
mod raycast;
mod ui;
mod voxel;

use bevy::prelude::*;
use player::PlayerPlugin;
use raycast::RaycastPlugin;
use ui::UiPlugin;
use voxel::VoxelPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.10, 0.14)))
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Voxworld".to_string(),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins((VoxelPlugin, PlayerPlugin, RaycastPlugin, UiPlugin))
        .run();
}
