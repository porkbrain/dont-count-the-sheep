use bevy::{render::view::RenderLayers, utils::Instant};
use common_visuals::camera::render_layer;

use crate::{
    cameras::BackgroundLightScene, hoshi, path::LevelPath, prelude::*,
};

/// When the mode is [`ClimateLightMode::Hot`], we deduct this much from the
/// score.
const HOT_DEDUCTION: usize = 80;
/// How often do we deduct from the score when the mode is
/// [`ClimateLightMode::Hot`].
const HOT_DEDUCTION_INTERVAL: Duration = from_millis(5_000);
/// When the mode is [`ClimateLightMode::Cold`], we deduct this much from the
/// score.
const COLD_DEDUCTION: usize = 100;
/// How often do we deduct from the score when the mode is
/// [`ClimateLightMode::Cold`].
const COLD_DEDUCTION_INTERVAL: Duration = from_millis(10_000);

#[derive(Component)]
pub(crate) struct Climate {
    path: LevelPath,
    current_path_since: Stopwatch,
    /// Timer for the rays of light.
    /// Allows us to pause the ray animation when the game is paused.
    rays_animation: Stopwatch,
    /// When was the mode changed and the mode itself.
    mode: (Instant, ClimateLightMode),
}

#[derive(Default, Clone, Copy)]
pub(crate) enum ClimateLightMode {
    #[default]
    Hot,
    Cold,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::LoadingMeditation), spawn)
            .add_systems(
                Update,
                toggle_mode
                    .run_if(on_event::<hoshi::ActionEvent>())
                    .run_if(in_state(GlobalGameState::InGameMeditation))
                    .after(hoshi::loading_special),
            )
            .add_systems(
                FixedUpdate,
                follow_curve
                    .run_if(in_state(GlobalGameState::InGameMeditation)),
            );
    }
}

fn spawn(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let climate = Climate::new();
    let climate_translation = {
        let (seg_index, seg_t) = climate.path_segment();
        let seg = &climate.path.segments()[seg_index];

        seg.position(seg_t).extend(zindex::CLIMATE)
    };

    cmd.spawn((
        climate,
        BackgroundLightScene,
        AngularVelocity::default(),
        SpatialBundle {
            transform: Transform::from_translation(climate_translation),
            ..default()
        },
    ))
    .with_children(|commands| {
        commands.spawn((
            RenderLayers::layer(render_layer::OBJ),
            SpriteBundle {
                texture: asset_server.load(assets::CLIMATE_DEFAULT),
                ..default()
            },
        ));
    });
}

/// Changes the mode of the climate on hoshi's special.
/// See readme for the game to understand what this means.
/// In short: we change light color, how score is deducted and how strong is
/// the ray on the Polpos.
fn toggle_mode(
    mut action: EventReader<hoshi::ActionEvent>,

    mut climate: Query<&mut Climate>,
    mut score: Query<&mut crate::ui::Score>,
) {
    let just_started_loading = action
        .read()
        .any(|e| matches!(e, hoshi::ActionEvent::FiredSpecial));

    if !just_started_loading {
        return;
    }

    let mut climate = climate.single_mut();
    let mut score = score.single_mut();

    let new_mode = !climate.mode.1;
    climate.mode = (Instant::now(), new_mode);
    score.set_deduction(new_mode.deduction());
    score.set_deduction_interval(new_mode.deduction_interval());
}

/// Polpos have something similar, but with some extra logic to change their
/// path.
fn follow_curve(
    mut climate: Query<(&mut Climate, &mut Transform)>,
    time: Res<Time>,
) {
    let (mut climate, mut transform) = climate.single_mut();

    climate.rays_animation.tick(time.delta());
    climate.current_path_since.tick(time.delta());

    let z = transform.translation.z;
    let (seg_index, seg_t) = climate.path_segment();

    let seg = &climate.path.segments()[seg_index];

    transform.translation = seg.position(seg_t).extend(z);
}

impl Climate {
    pub(crate) fn mode(&self) -> ClimateLightMode {
        self.mode.1
    }

    fn new() -> Self {
        Self {
            path: LevelPath::InfinitySign,
            current_path_since: Stopwatch::default(),
            rays_animation: Stopwatch::default(),
            mode: (Instant::now(), default()),
        }
    }

    fn path_segment(&self) -> (usize, f32) {
        self.path.segment(&self.current_path_since.elapsed())
    }
}

impl ClimateLightMode {
    pub(crate) fn deduction(self) -> usize {
        match self {
            ClimateLightMode::Hot => HOT_DEDUCTION,
            ClimateLightMode::Cold => COLD_DEDUCTION,
        }
    }

    pub(crate) fn deduction_interval(self) -> Duration {
        match self {
            ClimateLightMode::Hot => HOT_DEDUCTION_INTERVAL,
            ClimateLightMode::Cold => COLD_DEDUCTION_INTERVAL,
        }
    }
}

impl std::ops::Not for ClimateLightMode {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ClimateLightMode::Hot => ClimateLightMode::Cold,
            ClimateLightMode::Cold => ClimateLightMode::Hot,
        }
    }
}
