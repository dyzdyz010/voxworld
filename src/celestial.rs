use bevy::camera::Exposure;
use bevy::light::{light_consts::lux, VolumetricLight, CascadeShadowConfigBuilder};
use bevy::prelude::*;
use std::f32::consts::PI;

/// Marker component for the sun light source
#[derive(Component)]
pub struct Sun;

/// Marker component for the moon light source
#[derive(Component)]
pub struct Moon;

/// Resource to control celestial body motion
#[derive(Resource)]
pub struct CelestialSettings {
    /// Whether celestial bodies are paused
    pub paused: bool,
    /// Rotation speed in radians per second (default: PI/10 ≈ 18 degrees/sec)
    pub rotation_speed: f32,
}

impl Default for CelestialSettings {
    fn default() -> Self {
        Self {
            paused: false,
            rotation_speed: PI / 10.0,
        }
    }
}

pub struct CelestialPlugin;

impl Plugin for CelestialPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CelestialSettings::default())
            .add_systems(Startup, setup_celestial_bodies)
            .add_systems(Update, (update_celestial_motion, update_auto_exposure));
    }
}

fn setup_celestial_bodies(mut commands: Commands) {
    // Configure cascade shadow map for sun
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 30.0,
        maximum_distance: 500.0,
        ..default()
    }
    .build();

    // Base rotation for celestial bodies (looking down at the world)
    let base_transform = Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z);

    // Sun - primary light source during day
    commands.spawn((
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        base_transform,
        VolumetricLight,
        cascade_shadow_config,
        Sun,
    ));

    // Moon - secondary light source during night
    // Real moonlight is ~1/400,000 of sunlight, but we use ~1/1000 for better game visibility
    // Moon starts 180 degrees opposite to the sun (rotated PI around X axis)
    let moon_transform = base_transform.with_rotation(base_transform.rotation * Quat::from_rotation_x(PI));

    commands.spawn((
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT / 1_000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.3,
            ..default()
        },
        moon_transform,
        Moon,
    ));

    // Ambient light for starlight/indirect sky illumination at night
    // Provides a subtle bluish fill light so the scene isn't completely black
    commands.spawn(AmbientLight {
        color: Color::srgb(0.6, 0.7, 1.0), // Slight blue tint for night sky
        brightness: 150.0,
        affects_lightmapped_meshes: true,
    });
}

fn update_celestial_motion(
    mut sun_query: Query<&mut Transform, (With<Sun>, Without<Moon>)>,
    mut moon_query: Query<&mut Transform, With<Moon>>,
    time: Res<Time>,
    settings: Res<CelestialSettings>,
) {
    if settings.paused {
        return;
    }

    let delta_rotation = time.delta_secs() * settings.rotation_speed;

    // Rotate both sun and moon around the X axis in the same direction
    // They maintain their 180 degree offset, so when sun sets, moon rises
    for mut sun_transform in &mut sun_query {
        sun_transform.rotate_x(-delta_rotation);
    }

    for mut moon_transform in &mut moon_query {
        moon_transform.rotate_x(-delta_rotation);
    }
}

/// Auto-exposure system that adjusts camera exposure based on sun position
/// - Daytime (sun above horizon): Higher EV100 (~13) for bright scenes
/// - Nighttime (sun below horizon): Lower EV100 (~5) to see in moonlight
fn update_auto_exposure(
    sun_query: Query<&Transform, With<Sun>>,
    mut camera_query: Query<&mut Exposure, With<Camera3d>>,
    time: Res<Time>,
) {
    let Ok(sun_transform) = sun_query.single() else {
        return;
    };

    // Get sun's direction (negative Z is "forward" for directional light)
    let sun_direction = sun_transform.forward().as_vec3();

    // Sun altitude: positive Y means sun is above horizon, negative means below
    // sun_direction points FROM the sun, so we negate Y to get sun's position in sky
    let sun_altitude = -sun_direction.y;

    // Exposure settings
    const DAY_EV100: f32 = 13.0;   // Bright daylight exposure
    const NIGHT_EV100: f32 = 5.0;  // Night exposure (lower = brighter image)
    const TRANSITION_SPEED: f32 = 2.0; // How fast exposure adapts

    // Calculate target exposure based on sun altitude
    // sun_altitude ranges from -1 (sun directly below) to 1 (sun directly above)
    // We use a smooth transition around the horizon (altitude = 0)
    let t = (sun_altitude * 2.0 + 1.0).clamp(0.0, 1.0); // Map [-0.5, 0.5] to [0, 1]
    let target_ev100 = NIGHT_EV100 + (DAY_EV100 - NIGHT_EV100) * t;

    // Smoothly interpolate current exposure toward target
    for mut exposure in &mut camera_query {
        let current = exposure.ev100;
        let delta = target_ev100 - current;
        exposure.ev100 = current + delta * (time.delta_secs() * TRANSITION_SPEED).min(1.0);
    }
}
