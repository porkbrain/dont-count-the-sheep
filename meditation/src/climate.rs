use bevy::{render::view::RenderLayers, time::Stopwatch};
use bevy_magic_light_2d::prelude::*;

use crate::{path::LevelPath, prelude::*};

#[derive(Component)]
pub(crate) struct Climate {
    path: LevelPath,
    current_path_since: Stopwatch,
}

#[derive(Component)]
struct ClimateLight;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (follow_curve, move_light_source));
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Climate::new())
        .insert(AngularVelocity::default())
        .insert(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                zindex::CLIMATE,
            )),
            ..default()
        })
        .insert(RenderLayers::layer(4)) // TODO
        .with_children(|commands| {
            commands.spawn(SpriteBundle {
                texture: asset_server.load("textures/climate/default.png"),
                ..default()
            });
        });

    commands
        .spawn(Name::new("lights")) // TODO: move to some light plugin
        .insert(SpatialBundle::default())
        .insert(RenderLayers::layer(4)) // TODO
        .with_children(|commands| {
            commands
                .spawn(ClimateLight)
                .insert(SpatialBundle { ..default() })
                .insert(OmniLightSource2D {
                    intensity: 1.0,
                    color: Color::rgb_u8(137, 79, 24),
                    jitter_intensity: 0.5,
                    falloff: Vec3::new(5.0, 5.0, 0.05),
                    ..default()
                });
        });
}

// TODO: dedup with distrations
pub(crate) fn follow_curve(
    mut climate: Query<(&mut Climate, &mut Transform)>,
    time: Res<Time>,
) {
    let (mut climate, mut transform) = climate.single_mut();

    climate.current_path_since.tick(time.delta());

    let z = transform.translation.z;
    let (seg_index, seg_t) = climate.path_segment();

    let seg = &climate.path.segments()[seg_index];

    transform.translation = seg.position(seg_t).extend(z);
}

/// TODO: light source should be a repeating animation to the beat
/// and the jumps + special should be reset
fn move_light_source(
    climate: Query<&Transform, (With<Climate>, Without<ClimateLight>)>,
    mut light: Query<&mut Transform, (With<ClimateLight>, Without<Climate>)>,
) {
    let climate = climate.single();
    let mut light = light.single_mut();

    light.translation = climate.translation;
}

impl Climate {
    fn new() -> Self {
        Self {
            path: LevelPath::InfinitySign,
            current_path_since: Stopwatch::default(),
        }
    }

    fn path_segment(&self) -> (usize, f32) {
        self.path.segment(&self.current_path_since.elapsed())
    }
}
