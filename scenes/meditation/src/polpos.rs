mod consts;
mod effects;
mod react;
mod spawner;
mod videos;

use common_physics::PoissonsEquationUpdateEvent;
use rand::random;
use videos::Video;

use self::consts::{JITTER_ON_HIT_INTENSITY, JITTER_ON_HIT_TIME_PENALTY};
use crate::{
    climate::{Climate, ClimateLightMode},
    gravity::Gravity,
    hoshi,
    path::LevelPath,
    prelude::*,
};

#[derive(Component)]
pub(crate) struct Polpo {
    video: Video,
    current_path_since: Stopwatch,
    path: LevelPath,
    transition_into: Option<LevelPath>,
    /// Applies random jitter in a direction.
    /// When Polpo cracks, it jitters in a direction of where the blow
    /// came if it was caused by the Hoshi.
    jitter: Vec2,
}

/// Anything that's spawned in this module has this entity.
/// Useful for despawning.
#[derive(Component)]
struct PolpoEntity;

#[derive(Event)]
struct PolpoDestroyedEvent {
    /// Which video was playing on the Polpo's screen.
    video: Video,
    /// Where the Polpo was when it was destroyed.
    at_translation: Vec2,
    /// Whether the Polpo was destroyed by the Hoshi special or by
    /// just accumulating cracks.
    by_special: bool,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::LoadingMeditation), spawn)
            .add_systems(OnExit(GlobalGameState::QuittingMeditation), despawn)
            .add_event::<PolpoDestroyedEvent>()
            .add_systems(
                Update,
                (
                    spawner::try_spawn_next,
                    follow_curve,
                    react::to_environment,
                    react::to_hoshi_special
                        .run_if(on_event::<hoshi::ActionEvent>())
                        .after(hoshi::loading_special),
                    destroyed
                        .run_if(on_event::<PolpoDestroyedEvent>())
                        .after(react::to_hoshi_special)
                        .after(react::to_environment),
                )
                    .run_if(in_state(GlobalGameState::InGameMeditation)),
            );
    }
}

fn spawn(mut cmd: Commands) {
    debug!("Spawning Spawner");

    cmd.insert_resource(spawner::Spawner::new());
}

fn despawn(mut cmd: Commands, entities: Query<Entity, With<PolpoEntity>>) {
    debug!("Despawning Spawner");

    cmd.remove_resource::<spawner::Spawner>();

    debug!("Despawning Polpos");
    for entity in entities.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

/// Climate has something similar, but without the level up logic.
fn follow_curve(
    climate: Query<&Climate>,
    mut polpos: Query<(&mut Polpo, &mut Transform)>,
    time: Res<Time>,
) {
    let dt_multiplier = match climate.single().mode() {
        ClimateLightMode::Hot => 1.25,
        ClimateLightMode::Cold => 0.75,
    };

    let dt = time.delta_seconds() * dt_multiplier;

    for (mut polpo, mut transform) in polpos.iter_mut() {
        polpo.current_path_since.tick(Duration::from_secs_f32(dt));

        let z = transform.translation.z;
        let (seg_index, seg_t) = polpo.path_segment();

        let at_least_one_lap = polpo.laps() > 0;
        let at_lap_beginning = seg_index == 0 && seg_t < 2. / 60.;
        let ready_to_transition = polpo.transition_into.is_some();

        if at_lap_beginning && at_least_one_lap && ready_to_transition {
            polpo.current_path_since.reset();
            polpo.path = polpo.transition_into.take().unwrap(); // safe ^
        } else if !ready_to_transition {
            // roll a dice to see if Polpo levels up
            // let should_level_up = rand::random::<f32>() < 0.8; // TODO
            let should_level_up = true;
            polpo.transition_into =
                Some(polpo.path.transition_into(should_level_up));
        }

        let seg = &polpo.path.segments()[seg_index];

        let random_sign = if random::<bool>() { 1.0 } else { -1.0 };
        let jitter = polpo.jitter * random_sign * JITTER_ON_HIT_INTENSITY;
        let expected_position = seg.position(seg_t) + jitter;

        transform.translation = expected_position.extend(z);
        // dampen the jitter over time
        polpo.jitter = {
            let j = polpo.jitter * (1.0 - JITTER_ON_HIT_TIME_PENALTY * dt);

            j.max(Vec2::ZERO)
        };
    }
}

/// Either Polpo is destroyed by the Hoshi's special or by accumulating
/// cracks.
fn destroyed(
    mut cmd: Commands,
    mut events: EventReader<PolpoDestroyedEvent>,
    mut gravity: EventWriter<PoissonsEquationUpdateEvent<Gravity>>,
    mut spawner: ResMut<spawner::Spawner>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,

    mut score: Query<&mut crate::ui::Score>,
) {
    for PolpoDestroyedEvent {
        video,
        at_translation,
        by_special,
    } in events.read()
    {
        debug!("Received Polpo destroyed event (special: {by_special})");

        let mut score = score.single_mut();
        // the further away the Polpo is, the more points it's worth
        *score += at_translation.length() as usize;
        // notify the spawner that the Polpo is gone
        spawner.despawn(*video);

        if !by_special {
            // TODO: some animation of the Polpo falling apart

            continue;
        }

        trace!("Spawning black hole");
        effects::black_hole::spawn(
            &mut cmd,
            &asset_server,
            &mut texture_atlases,
            &mut gravity,
            *at_translation,
        );
    }
}

impl Polpo {
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
            jitter: Vec2::ZERO,
        }
    }
}
