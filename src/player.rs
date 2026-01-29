use bevy::anti_alias::fxaa::Fxaa;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium, ScreenSpaceReflections};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

use crate::ui::MenuState;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component, Debug, Default)]
pub struct LookAngles {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Resource)]
pub struct PlayerSettings {
    pub move_speed: f32,
    pub look_sensitivity: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerSettings {
            move_speed: 6.5,
            look_sensitivity: 0.0025,
        })
        .add_systems(Startup, setup_player)
        .add_systems(Update, (player_look, player_move));
    }
}

fn setup_player(
    mut commands: Commands,
    mut cursor_options: Single<&mut CursorOptions>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    let yaw = 0.0;
    let pitch = -0.15;
    let rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);

    // Camera with atmosphere components
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 20.0).with_rotation(rotation),
        PlayerCamera,
        LookAngles { yaw, pitch },
        // Earthlike atmosphere
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings::default(),
        // Exposure compensation for bright atmospheric lighting
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        // Bloom gives the sun a much more natural look
        Bloom::NATURAL,
        // Enables the atmosphere to drive reflections and ambient lighting
        AtmosphereEnvironmentMapLight::default(),
        VolumetricFog {
            ambient_intensity: 0.0,
            ..default()
        },
        Msaa::Off,
        Fxaa::default(),
        ScreenSpaceReflections::default(),
    ));

    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

fn player_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<(&mut Transform, &mut LookAngles), With<PlayerCamera>>,
    settings: Res<PlayerSettings>,
    menu_state: Res<MenuState>,
) {
    if menu_state.open {
        return;
    }
    let delta = mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }
    let Ok((mut transform, mut angles)) = query.single_mut() else {
        return;
    };
    angles.yaw -= delta.x * settings.look_sensitivity;
    angles.pitch = (angles.pitch - delta.y * settings.look_sensitivity).clamp(-1.54, 1.54);
    let yaw = Quat::from_axis_angle(Vec3::Y, angles.yaw);
    let pitch = Quat::from_axis_angle(Vec3::X, angles.pitch);
    transform.rotation = yaw * pitch;
}

fn player_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<PlayerCamera>>,
    settings: Res<PlayerSettings>,
    menu_state: Res<MenuState>,
) {
    if menu_state.open {
        return;
    }
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let forward = transform.forward().as_vec3();
    let right = transform.right().as_vec3();
    let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    let mut input = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        input += forward_flat;
    }
    if keys.pressed(KeyCode::KeyS) {
        input -= forward_flat;
    }
    if keys.pressed(KeyCode::KeyA) {
        input -= right_flat;
    }
    if keys.pressed(KeyCode::KeyD) {
        input += right_flat;
    }
    if keys.pressed(KeyCode::Space) {
        input += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        input -= Vec3::Y;
    }

    if input == Vec3::ZERO {
        return;
    }

    transform.translation += input.normalize_or_zero() * settings.move_speed * time.delta_secs();
}
