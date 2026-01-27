mod player;
mod raycast;
mod ui;
mod voxel;

use std::f32::consts::PI;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::input::keyboard::KeyCode;
use bevy::pbr::{AtmosphereMode, AtmosphereSettings};
use bevy::prelude::*;
use bevy::camera::Exposure;
use player::PlayerPlugin;
use raycast::RaycastPlugin;
use ui::UiPlugin;
use voxel::{VoxelPlugin, WorldSeed};

#[derive(Resource, Default)]
struct GameState {
    paused: bool,
}

fn main() {
    // Parse seed from command line or environment variable
    let seed = parse_seed();

    App::new()
        .insert_resource(seed)
        .insert_resource(GameState::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxworld".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            VoxelPlugin,
            PlayerPlugin,
            RaycastPlugin,
            UiPlugin,
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, print_controls)
        .add_systems(Update, (dynamic_scene, atmosphere_controls))
        .run();
}

fn print_controls() {
    println!("=== Voxworld Controls ===");
    println!("  WASD       - Move");
    println!("  Space      - Move up");
    println!("  Shift      - Move down");
    println!("  Mouse      - Look around");
    println!("  Esc        - Pause menu");
    println!("  F3         - Toggle debug overlay");
    println!();
    println!("=== Atmosphere Controls ===");
    println!("  1          - Switch to lookup texture rendering method");
    println!("  2          - Switch to raymarched rendering method");
    println!("  P          - Pause/Resume sun motion");
    println!("  Up/Down    - Increase/Decrease exposure");
}

fn atmosphere_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut atmosphere_settings: Query<&mut AtmosphereSettings>,
    mut game_state: ResMut<GameState>,
    mut camera_exposure: Query<&mut Exposure, With<Camera3d>>,
    time: Res<Time>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        for mut settings in &mut atmosphere_settings {
            settings.rendering_method = AtmosphereMode::LookupTexture;
            println!("Switched to lookup texture rendering method");
        }
    }

    if keyboard_input.just_pressed(KeyCode::Digit2) {
        for mut settings in &mut atmosphere_settings {
            settings.rendering_method = AtmosphereMode::Raymarched;
            println!("Switched to raymarched rendering method");
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyP) {
        game_state.paused = !game_state.paused;
        println!("Sun motion: {}", if game_state.paused { "PAUSED" } else { "RESUMED" });
    }

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for mut exposure in &mut camera_exposure {
            exposure.ev100 -= time.delta_secs() * 2.0;
        }
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) {
        for mut exposure in &mut camera_exposure {
            exposure.ev100 += time.delta_secs() * 2.0;
        }
    }
}

fn dynamic_scene(
    mut suns: Query<&mut Transform, With<DirectionalLight>>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    // Only rotate the sun if motion is not paused
    if !game_state.paused {
        suns.iter_mut()
            .for_each(|mut tf| tf.rotate_x(-time.delta_secs() * PI / 10.0));
    }
}

fn parse_seed() -> WorldSeed {
    // Check command line arguments: --seed <value> or -s <value>
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if (args[i] == "--seed" || args[i] == "-s") && i + 1 < args.len() {
            let seed_str = &args[i + 1];
            // Try to parse as number first
            if let Ok(num) = seed_str.parse::<u32>() {
                info!("Using seed from command line: {}", num);
                return WorldSeed::new(num);
            } else {
                // Use string as seed
                info!("Using string seed from command line: {}", seed_str);
                return WorldSeed::from_string(seed_str);
            }
        }
    }

    // Check environment variable
    if let Ok(seed_str) = std::env::var("VOXWORLD_SEED") {
        if let Ok(num) = seed_str.parse::<u32>() {
            info!("Using seed from environment: {}", num);
            return WorldSeed::new(num);
        } else {
            info!("Using string seed from environment: {}", seed_str);
            return WorldSeed::from_string(&seed_str);
        }
    }

    // Generate random seed based on current time
    let random_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u32)
        .unwrap_or(12345);

    info!("Using random seed: {}", random_seed);
    WorldSeed::new(random_seed)
}
