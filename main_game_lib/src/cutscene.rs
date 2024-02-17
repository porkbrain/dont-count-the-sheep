//! Utility for creating cutscenes.
//!
//! The cutscene is a state machine that executes a sequence of steps.
//! Each step corresponds to a system.
//! Next system is scheduled at the end of the previous one.
//! A system can repeat itself over and over when waiting for some condition
//! such as timer or existence/non-existence of a resource etc.
//!
//! Note that several systems can be scheduled within the same frame, one after
//! another.
//! Specifically, those systems that don't wait for anything.

use std::{sync::OnceLock, time::Duration};

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, ecs::system::SystemId,
    math::vec2, prelude::*, render::view::RenderLayers, time::Stopwatch,
};
use bevy_grid_squared::{GridDirection, Square};
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_store::GlobalStore;
use common_story::portrait_dialog::{DialogRoot, PortraitDialog};
use common_top_down::{Actor, ActorTarget, Player};
use common_visuals::{
    camera::{
        order, render_layer, PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH,
        PIXEL_ZOOM,
    },
    AtlasAnimation, AtlasAnimationTimer, BeginInterpolationEvent,
};

use crate::{
    prelude::from_millis, GlobalGameState, GlobalGameStateTransition,
    GlobalGameStateTransitionStack,
};

const LETTERBOXING_FADE_IN_DURATION: Duration = from_millis(500);
const LETTERBOXING_FADE_OUT_DURATION: Duration = from_millis(250);
const LETTERBOXING_QUAD_HEIGHT: f32 = 50.0;
const LETTERBOXING_TOP_QUAD_INITIAL_POS: Vec2 = vec2(
    0.,
    PIXEL_VISIBLE_HEIGHT / 2.0 + LETTERBOXING_QUAD_HEIGHT / 2.0,
);
const LETTERBOXING_TOP_QUAD_TARGET_POS: Vec2 = vec2(
    0.0,
    PIXEL_VISIBLE_HEIGHT / 2.0 - LETTERBOXING_QUAD_HEIGHT / 2.0,
);
const LETTERBOXING_BOTTOM_QUAD_INITIAL_POS: Vec2 = vec2(
    0.,
    -PIXEL_VISIBLE_HEIGHT / 2.0 - LETTERBOXING_QUAD_HEIGHT / 2.0,
);
const LETTERBOXING_BOTTOM_QUAD_TARGET_POS: Vec2 = vec2(
    0.0,
    -PIXEL_VISIBLE_HEIGHT / 2.0 + LETTERBOXING_QUAD_HEIGHT / 2.0,
);

/// Will be true if there's a cutscene playing.
pub fn in_cutscene() -> impl FnMut(Option<Res<Cutscene>>) -> bool {
    move |cutscene| cutscene.is_some()
}

/// Will be true if there's no cutscene playing.
pub fn not_in_cutscene() -> impl FnMut(Option<Res<Cutscene>>) -> bool {
    move |cutscene| cutscene.is_none()
}

/// A trait for creating cutscenes.
pub trait IntoCutscene {
    /// A cutscene is a sequence of steps.
    fn sequence(self) -> Vec<CutsceneStep>;

    /// If true, the cutscene will be letterboxed with two black bars: above and
    /// below the screen.
    /// A camera and two quads are spawned for this purpose.
    /// They are despawned when the cutscene ends.
    ///
    /// Make sure you run the [`common_visuals::systems::smoothly_translate`]
    /// system.
    ///
    /// <https://en.wikipedia.org/wiki/Letterboxing_(filming)>
    fn has_letterboxing() -> bool {
        false
    }

    /// Helper function that spawns the cutscene.
    /// You don't need to do anything beyond this.
    fn spawn(self, cmd: &mut Commands)
    where
        Self: Sized,
    {
        spawn_cutscene(cmd, self);
    }
}

/// There can be at most one cutscene running at a time.
/// Cutscene can be connected with a dialog though.
#[derive(Resource, Reflect)]
pub struct Cutscene {
    /// Whether the cutscene is loaded.
    /// This is set to true on the first frame that the cutscene is a resource.
    is_loaded: bool,
    /// The sequence of steps that the cutscene will execute.
    /// Current step is tracked by `sequence_index`.
    sequence: Vec<CutsceneStep>,
    /// When the cutscene reaches the end, we remove this resource and clean up
    /// eventual entities spawned by the cutscene (e.g. letterboxing quads and
    /// camera).
    sequence_index: usize,
    /// Many steps require some sort of timing.
    /// This is the stopwatch that is used to measure time.
    /// It resets after every step.
    stopwatch: Stopwatch,
    /// If letterboxing was spawned, then this contains the three entities that
    /// must be despawned when the cutscene ends.
    letterboxing_entities: Option<[Entity; 3]>,
}

/// What are the possible operations that a cutscene can perform?
#[derive(Reflect, Clone, strum::Display)]
pub enum CutsceneStep {
    /// (condition, on_true, on_false)
    IfTrueThisElseThat(
        fn(&GlobalStore) -> bool,
        Box<Vec<CutsceneStep>>,
        Box<Vec<CutsceneStep>>,
    ),

    /// Simple await until this duration passes.
    Sleep(Duration),
    /// Removes the [`Player`] component from the player entity.
    /// Ideal for the first step of the cutscene.
    RemovePlayerComponent(Entity),
    /// Inserts the [`Player`] component to the player entity.
    /// This gives the player back control, ideal for the last step of the
    /// cutscene.
    AddPlayerComponent(Entity),
    /// Inserts [`AtlasAnimationTimer`] component to the given entity.
    /// This typically is used to start an animation when the entity has
    /// [`AtlasAnimation`] component.
    InsertAtlasAnimationTimerTo {
        /// Will be used with [`Commands`] to insert the component.
        entity: Entity,
        /// How long should the animation last.
        duration: Duration,
        /// This is typically [`TimerMode::Repeating`].
        mode: TimerMode,
    },
    /// An entity that has [`AtlasAnimation`] component will have its
    /// `reversed` flag toggled.
    ReverseAtlasAnimation(Entity),
    /// Waits until an entity that has [`AtlasAnimation`] component reaches the
    /// end. Whether the animation is played in reverse or not is taken
    /// into account.
    WaitUntilAtlasAnimationEnds(Entity),
    /// Transitions to the given state.
    /// Typically, this despawns the whole scene so it can be the last step
    /// in the cutscene.
    ChangeGlobalState {
        /// The state to change to.
        /// Typically this would be a quitting state so that we transition into
        /// a new scene.
        to: GlobalGameState,
        /// The transition to use.
        /// Pair this with the `to` field to guide the game into the next
        /// scene.
        with: GlobalGameStateTransition,
        /// If provided then this resource is inserted.
        loading_screen: Option<LoadingScreenSettings>,
        /// If true loading screen state is changed.
        /// It's important that the state is not changed twice.
        change_loading_screen_state_to_start: bool,
    },
    /// Gives the actor a new target to walk to.
    /// No path finding or iteration is done here, the actor will transition
    /// to the new target immediately.
    BeginSimpleWalkTo {
        /// Must be an [`Actor`].
        with: Entity,
        /// The target square actor walks to.
        /// Will be used to construct [`ActorTarget`].
        square: Square,
        /// Once the current target is reached, we can plan the next one.
        /// Will be used to construct [`ActorTarget`].
        planned: Option<(Square, GridDirection)>,
        /// If provided then the actor will walk with this speed.
        /// If not provided, the default speed will be used.
        step_time: Option<Duration>,
    },
    /// Wait till [`Actor`] entity has `walking_to` set to [`None`].
    WaitUntilActorAtRest(Entity),
    /// Starts given dialog.
    BeginPortraitDialog(DialogRoot),
    /// Waits until there is no portrait dialog resource.
    WaitForPortraitDialogToEnd,
    /// Inserts [`SmoothTranslation`] to the given entity.
    /// The entity must have [`Transform`] component and the
    /// [`common_visuals::systems::smoothly_translate`] system must be run.
    /// Also, this system does not wait for the translation to end.
    /// It just starts it.
    BeginMovingEntity {
        /// Which entity to move.
        /// Must have [`Transform`] component.
        who: Entity,
        /// Where to go.
        to: Destination,
        /// How long should this translation take.
        over: Duration,
        /// By default linear interpolation is used.
        animation_curve: Option<CubicSegment<Vec2>>,
    },
}

/// Marks a destination.
#[derive(Reflect, Clone, Copy, strum::Display)]
pub enum Destination {
    /// Given by the [`Transform`] component.
    Entity(Entity),
    /// Specific position in the world.
    Position(Vec2),
}

#[derive(Component)]
struct LetterboxingCamera;
#[derive(Component)]
struct LetterboxingTopQuad;
#[derive(Component)]
struct LetterboxingBottomQuad;

/// Registers all the necessary systems that handle cutscene logic.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let w = &mut app.world;

        let systems = CutsceneSystems::register_systems(w);

        CUTSCENE_SYSTEMS.get_or_init(|| systems);

        app.add_systems(First, schedule_current_step.run_if(in_cutscene()));
    }
}

/// Spawns the given cutscene.
pub fn spawn_cutscene<Scene: IntoCutscene>(
    cmd: &mut Commands,
    cutscene: Scene,
) {
    let sequence = cutscene.sequence();
    debug_assert!(!sequence.is_empty());
    let has_letterboxing = Scene::has_letterboxing();

    let letterboxing_entities = if has_letterboxing {
        // Render layers will be inserted once the cutscene is loaded.
        // This is to prevent a flicker when the quads are rendered but the
        // pixel zoom is not yet set.
        let camera = cmd
            .spawn((
                Name::new("Letterboxing: camera"),
                LetterboxingCamera,
                PixelZoom::Fixed(PIXEL_ZOOM),
                PixelViewport,
                Camera2dBundle {
                    camera: Camera {
                        hdr: true,
                        order: order::CUTSCENE_LETTERBOXING,
                        ..default()
                    },
                    camera_2d: Camera2d {
                        clear_color: ClearColorConfig::None,
                    },
                    ..default()
                },
            ))
            .id();

        const TOP_QUAD_INITIAL_POS: Vec2 = vec2(0., PIXEL_VISIBLE_HEIGHT / 2.0);
        const TOP_QUAD_TARGET_POS: Vec2 = vec2(
            0.0,
            PIXEL_VISIBLE_HEIGHT / 2.0 - LETTERBOXING_QUAD_HEIGHT / 2.0,
        );

        let top = cmd
            .spawn((
                Name::new("Letterboxing: top quad"),
                RenderLayers::layer(render_layer::CUTSCENE_LETTERBOXING),
                LetterboxingTopQuad,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(
                            PIXEL_VISIBLE_WIDTH,
                            LETTERBOXING_QUAD_HEIGHT,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(
                        TOP_QUAD_INITIAL_POS.extend(0.0),
                    ),
                    ..default()
                },
            ))
            .id();
        BeginInterpolationEvent::of_translation(
            top,
            Some(TOP_QUAD_INITIAL_POS),
            TOP_QUAD_TARGET_POS,
        )
        .over(LETTERBOXING_FADE_IN_DURATION)
        .insert_to(cmd);

        let bottom = cmd
            .spawn((
                Name::new("Letterboxing: bottom quad"),
                LetterboxingBottomQuad,
                RenderLayers::layer(render_layer::CUTSCENE_LETTERBOXING),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(
                            PIXEL_VISIBLE_WIDTH,
                            LETTERBOXING_QUAD_HEIGHT,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(
                        LETTERBOXING_BOTTOM_QUAD_INITIAL_POS.extend(0.0),
                    ),
                    ..default()
                },
            ))
            .id();
        BeginInterpolationEvent::of_translation(
            top,
            Some(LETTERBOXING_BOTTOM_QUAD_INITIAL_POS),
            LETTERBOXING_BOTTOM_QUAD_TARGET_POS,
        )
        .over(LETTERBOXING_FADE_IN_DURATION)
        .insert_to(cmd);

        Some([camera, top, bottom])
    } else {
        None
    };

    let cutscene = Cutscene {
        sequence,
        sequence_index: 0,
        stopwatch: Stopwatch::new(),
        letterboxing_entities,
        is_loaded: false, // will be on next frame
    };

    // this will be picked up by `schedule_current_step` system
    cmd.insert_resource(cutscene);
}

/// Ticks the stopwatch and schedules the system for the current step of the
/// cutscene.
/// This must run only if there's a cutscene resource.
fn schedule_current_step(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
    time: Res<Time>,

    camera: Query<Entity, With<LetterboxingCamera>>,
) {
    if !cutscene.is_loaded {
        cutscene.is_loaded = true;

        if cutscene.letterboxing_entities.is_some() {
            cmd.entity(camera.single()).insert(RenderLayers::layer(
                render_layer::CUTSCENE_LETTERBOXING,
            ));
        }
    }

    cutscene.stopwatch.tick(time.delta());
    cutscene.schedule_current_step(&mut cmd);
}

impl Cutscene {
    fn schedule_next_step_or_despawn(&mut self, cmd: &mut Commands) {
        if self.sequence_index >= self.sequence.len() - 1 {
            cmd.remove_resource::<Cutscene>();

            if let Some(entities) = self.letterboxing_entities.take() {
                // smoothly animate quads out and when down, despawn everything

                BeginInterpolationEvent::of_translation(
                    entities[2],
                    Some(LETTERBOXING_BOTTOM_QUAD_TARGET_POS),
                    LETTERBOXING_BOTTOM_QUAD_INITIAL_POS,
                )
                .over(LETTERBOXING_FADE_OUT_DURATION)
                .insert_to(cmd);

                BeginInterpolationEvent::of_translation(
                    entities[1],
                    Some(LETTERBOXING_TOP_QUAD_TARGET_POS),
                    LETTERBOXING_TOP_QUAD_INITIAL_POS,
                )
                .over(LETTERBOXING_FADE_OUT_DURATION)
                .when_finished_do(move |cmd: &mut Commands| {
                    cmd.entity(entities[0]).despawn();
                    cmd.entity(entities[1]).despawn();
                    cmd.entity(entities[2]).despawn();
                })
                .insert_to(cmd);
            }
        } else {
            self.stopwatch.reset();
            self.sequence_index += 1;
            self.schedule_current_step(cmd);
        }
    }

    fn schedule_current_step(&self, cmd: &mut Commands) {
        cmd.run_system(self.current_step_system_id())
    }

    fn current_step_system_id(&self) -> SystemId {
        system_id(&self.sequence[self.sequence_index])
    }
}

static CUTSCENE_SYSTEMS: OnceLock<CutsceneSystems> = OnceLock::new();

struct CutsceneSystems {
    if_true_this_else_that: SystemId,
    remove_player_component: SystemId,
    add_player_component: SystemId,
    sleep: SystemId,
    insert_atlas_animation_timer_to: SystemId,
    change_global_state: SystemId,
    begin_simple_walk_to: SystemId,
    wait_until_actor_at_rest: SystemId,
    begin_portrait_dialog: SystemId,
    wait_for_portrait_dialog_to_end: SystemId,
    reverse_atlas_animation: SystemId,
    wait_until_atlas_animation_ends: SystemId,
    begin_moving_entity: SystemId,
}

impl CutsceneSystems {
    fn register_systems(w: &mut World) -> Self {
        Self {
            if_true_this_else_that: w.register_system(if_true_this_else_that),
            remove_player_component: w.register_system(remove_player_component),
            add_player_component: w.register_system(add_player_component),
            sleep: w.register_system(sleep),
            insert_atlas_animation_timer_to: w
                .register_system(insert_atlas_animation_timer_to),
            change_global_state: w.register_system(change_global_state),
            begin_simple_walk_to: w.register_system(begin_simple_walk_to),
            wait_until_actor_at_rest: w
                .register_system(wait_until_actor_at_rest),
            begin_portrait_dialog: w.register_system(begin_portrait_dialog),
            wait_for_portrait_dialog_to_end: w
                .register_system(wait_for_portrait_dialog_to_end),
            reverse_atlas_animation: w.register_system(reverse_atlas_animation),
            wait_until_atlas_animation_ends: w
                .register_system(wait_until_atlas_animation_ends),
            begin_moving_entity: w.register_system(begin_moving_entity),
        }
    }
}

fn system_id(step: &CutsceneStep) -> SystemId {
    use CutsceneStep::*;

    debug_assert!(
        CUTSCENE_SYSTEMS.get().is_some(),
        "Cutscene systems not initialized"
    );

    let s = CUTSCENE_SYSTEMS.get().unwrap();
    match step {
        IfTrueThisElseThat(_, _, _) => s.if_true_this_else_that,
        RemovePlayerComponent(_) => s.remove_player_component,
        AddPlayerComponent(_) => s.add_player_component,
        Sleep(_) => s.sleep,
        InsertAtlasAnimationTimerTo { .. } => s.insert_atlas_animation_timer_to,
        ChangeGlobalState { .. } => s.change_global_state,
        BeginSimpleWalkTo { .. } => s.begin_simple_walk_to,
        WaitUntilActorAtRest(_) => s.wait_until_actor_at_rest,
        BeginPortraitDialog(_) => s.begin_portrait_dialog,
        WaitForPortraitDialogToEnd => s.wait_for_portrait_dialog_to_end,
        ReverseAtlasAnimation(_) => s.reverse_atlas_animation,
        WaitUntilAtlasAnimationEnds(_) => s.wait_until_atlas_animation_ends,
        BeginMovingEntity { .. } => s.begin_moving_entity,
    }
}

fn remove_player_component(mut cmd: Commands, mut cutscene: ResMut<Cutscene>) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::RemovePlayerComponent(entity) = &step else {
        panic!("Expected RemovePlayerComponent step, got {step}");
    };

    cmd.entity(*entity).remove::<Player>();

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn add_player_component(mut cmd: Commands, mut cutscene: ResMut<Cutscene>) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::AddPlayerComponent(entity) = &step else {
        panic!("Expected AddPlayerComponent step, got {step}");
    };

    cmd.entity(*entity).insert(Player);

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn sleep(mut cmd: Commands, mut cutscene: ResMut<Cutscene>) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::Sleep(duration) = &step else {
        panic!("Expected Sleep step, got {step}");
    };

    if cutscene.stopwatch.elapsed() >= *duration {
        cutscene.schedule_next_step_or_despawn(&mut cmd);
    }
}

fn insert_atlas_animation_timer_to(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::InsertAtlasAnimationTimerTo {
        entity,
        duration,
        mode,
    } = &step
    else {
        panic!("Expected InsertAnimationTimerTo step, got {step}");
    };

    cmd.entity(*entity)
        .insert(AtlasAnimationTimer::new(*duration, *mode));
    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn change_global_state(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
    mut transition: ResMut<GlobalGameStateTransitionStack>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::ChangeGlobalState {
        to,
        with,
        loading_screen,
        change_loading_screen_state_to_start,
    } = &step
    else {
        panic!("Expected ChangeGlobalState step, got {step}");
    };

    if let Some(setting) = loading_screen {
        trace!("[ChangeGlobalState] Inserting loading screen settings");
        cmd.insert_resource(setting.clone());
    }

    if *change_loading_screen_state_to_start {
        trace!("[ChangeGlobalState] Setting loading screen state to start");
        next_loading_screen_state.set(common_loading_screen::start_state());
    }

    next_state.set(*to);
    transition.push(*with);

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn begin_simple_walk_to(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,

    mut actors: Query<&mut Actor>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::BeginSimpleWalkTo {
        with,
        square,
        planned,
        step_time,
    } = &step
    else {
        panic!("Expected SimpleWalkTo step, got {step}");
    };

    if let Ok(mut actor) = actors.get_mut(*with) {
        if let Some(step_time) = step_time {
            actor.step_time = *step_time;
        }
        actor.walking_to = Some(ActorTarget {
            square: *square,
            planned: *planned,
            since: Stopwatch::new(),
        });
    }

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn wait_until_actor_at_rest(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,

    actors: Query<&Actor>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::WaitUntilActorAtRest(entity) = &step else {
        panic!("Expected WaitUntilActorAtRest step, got {step}");
    };

    if let Ok(actor) = actors.get(*entity) {
        if actor.walking_to.is_none() {
            cutscene.schedule_next_step_or_despawn(&mut cmd);
        }
    }
}

fn begin_portrait_dialog(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
    asset_server: Res<AssetServer>,
    global_store: Res<GlobalStore>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::BeginPortraitDialog(dialog) = &step else {
        panic!("Expected BeginDialog step, got {step}");
    };

    dialog.spawn(&mut cmd, &asset_server, &global_store);

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn wait_for_portrait_dialog_to_end(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
    dialog: Option<Res<PortraitDialog>>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::WaitForPortraitDialogToEnd = &step else {
        panic!("Expected WaitForPortraitDialogToEnd step, got {step}");
    };

    if dialog.is_none() {
        cutscene.schedule_next_step_or_despawn(&mut cmd);
    }
}

fn if_true_this_else_that(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
    store: Res<GlobalStore>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::IfTrueThisElseThat(cond, on_true, on_false) = &step
    else {
        panic!("Expected IfTrueThisElseThat step, got {step}");
    };

    let sequence = if cond(&store) {
        *on_true.clone()
    } else {
        *on_false.clone()
    };

    cutscene.sequence_index = 0;
    cutscene.sequence = sequence;

    cutscene.schedule_current_step(&mut cmd);
}

fn reverse_atlas_animation(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,

    mut animations: Query<&mut AtlasAnimation>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::ReverseAtlasAnimation(entity) = &step else {
        panic!("Expected ReverseAnimation step, got {step}");
    };

    if let Ok(mut animation) = animations.get_mut(*entity) {
        animation.reversed = !animation.reversed;
    }

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn wait_until_atlas_animation_ends(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,

    animations: Query<Option<&AtlasAnimationTimer>, With<AtlasAnimation>>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::WaitUntilAtlasAnimationEnds(entity) = &step else {
        panic!("Expected WaitUntilAnimationEnds step, got {step}");
    };

    if let Ok(timer) = animations.get(*entity) {
        if timer.is_none() {
            cutscene.schedule_next_step_or_despawn(&mut cmd);
        }
    }
}

fn begin_moving_entity(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,

    transforms: Query<&Transform>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::BeginMovingEntity {
        who,
        to,
        over,
        animation_curve,
    } = &step
    else {
        panic!("Expected MoveEntityTranslationTo step, got {step}");
    };

    if let Ok(transform) = transforms.get(*who) {
        let from = transform.translation.truncate();

        let to = match to {
            Destination::Entity(entity) => transforms
                .get(*entity)
                .expect("Entity in cutscene must exist") // SOFTEN
                .translation
                .truncate(),
            Destination::Position(pos) => *pos,
        };

        BeginInterpolationEvent::of_translation(*who, Some(from), to)
            .over(*over)
            .with_animation_opt_curve(animation_curve.clone())
            .insert_to(&mut cmd);
    }

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}
