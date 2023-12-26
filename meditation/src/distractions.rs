mod black_hole;
mod consts;
mod react;
mod spawner;
mod videos;

use bevy::time::Stopwatch;
use common_physics::PoissonsEquationUpdateEvent;

use crate::path::LevelPath;
use crate::{gravity::Gravity, prelude::*, weather};
use videos::Video;

#[derive(Component)]
pub(crate) struct Distraction {
    video: Video,
    current_path_since: Stopwatch,
    path: LevelPath,
    transition_into: Option<LevelPath>,
}
#[derive(Component)]
struct DistractionOccluder;

/// Anything that's spawned in this module has this entity.
/// Useful for despawning.
#[derive(Component)]
struct DistractionEntity;

#[derive(Event)]
struct DistractionDestroyedEvent {
    /// Which video was playing on the distraction.
    video: Video,
    /// Where the distraction was when it was destroyed.
    at_translation: Vec2,
    /// Whether the distraction was destroyed by the weather special or by
    /// just accumulating cracks.
    by_special: bool,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::MeditationLoading), spawn)
            .add_systems(OnEnter(GlobalGameState::MeditationQuitting), despawn)
            .add_event::<DistractionDestroyedEvent>()
            .add_systems(
                Update,
                (
                    spawner::try_spawn_next,
                    follow_curve,
                    react::to_environment,
                    react::to_weather_special.after(weather::loading_special),
                    destroyed
                        .after(react::to_weather_special)
                        .after(react::to_environment),
                )
                    .run_if(in_state(GlobalGameState::MeditationInGame)),
            );
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

fn spawn(mut commands: Commands) {
    debug!("Spawning Spawner");

    commands.insert_resource(spawner::Spawner::new());
}

fn despawn(
    mut commands: Commands,
    entities: Query<Entity, With<DistractionEntity>>,
) {
    debug!("Despawning Spawner");

    commands.remove_resource::<spawner::Spawner>();

    debug!("Despawning distractions");
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Climate has something similar, but without the level up logic.
fn follow_curve(
    mut distraction: Query<(&mut Distraction, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut distraction, mut transform) in distraction.iter_mut() {
        distraction.current_path_since.tick(time.delta());

        let z = transform.translation.z;
        let (seg_index, seg_t) = distraction.path_segment();

        let at_least_one_lap = distraction.laps() > 0;
        let at_lap_beginning = seg_index == 0 && seg_t < 2. / 60.;
        let ready_to_transition = distraction.transition_into.is_some();

        if at_lap_beginning && at_least_one_lap && ready_to_transition {
            distraction.current_path_since.reset();
            distraction.path = distraction.transition_into.take().unwrap();
        } else if !ready_to_transition {
            // roll a dice to see if distraction levels up
            // let should_level_up = rand::random::<f32>() < 0.8; // TODO
            let should_level_up = true;
            distraction.transition_into =
                Some(distraction.path.transition_into(should_level_up));
        }

        let seg = &distraction.path.segments()[seg_index];

        transform.translation = seg.position(seg_t).extend(z);
    }
}

/// Either distraction is destroyed by the weather special or by accumulating
/// cracks.
fn destroyed(
    mut score: Query<&mut crate::ui::Score>,
    mut spawner: ResMut<spawner::Spawner>,
    mut events: EventReader<DistractionDestroyedEvent>,
    mut gravity: EventWriter<PoissonsEquationUpdateEvent<Gravity>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if events.is_empty() {
        return;
    }

    let mut score = score.single_mut();

    for DistractionDestroyedEvent {
        video,
        at_translation,
        by_special,
    } in events.read()
    {
        debug!("Received distraction destroyed event (special: {by_special})");

        // the further away the distraction is, the more points it's worth
        *score += at_translation.length() as usize;
        // notify the spawner that the distraction is gone
        spawner.despawn(*video);

        if !by_special {
            // TODO: some animation of the distraction falling apart

            continue;
        }

        trace!("Spawning black hole");
        black_hole::spawn(
            &mut commands,
            &asset_server,
            &mut texture_atlases,
            &mut gravity,
            *at_translation,
        );
    }
}

impl Distraction {
    fn path_segment(&self) -> (usize, f32) {
        self.path.segment(&self.current_path_since.elapsed())
    }

    fn laps(&self) -> usize {
        (self.current_path_since.elapsed_secs() / self.path.total_path_time())
            as usize
    }

    fn new(video: Video) -> Self {
        Self {
            video,
            path: LevelPath::random_intro(),
            current_path_since: Stopwatch::new(),
            transition_into: None,
        }
    }
}
