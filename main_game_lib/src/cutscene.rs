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

pub mod enter_an_elevator;
pub mod enter_dark_door;

use std::sync::OnceLock;

use bevy::{ecs::system::SystemId, prelude::*, render::view::RenderLayers};
use bevy_grid_squared::{GridDirection, Square};
use common_ext::QueryExt;
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_store::GlobalStore;
use common_story::dialog::{
    self, fe::portrait::PortraitDialog, StartDialogWhenLoaded,
};
use common_visuals::{
    camera::{order, render_layer, MainCamera},
    AtlasAnimation, AtlasAnimationStep, AtlasAnimationTimer,
    BeginInterpolationEvent,
};

use self::top_down::cameras::{ManualControl, SyncWithPlayer};
use crate::{
    prelude::*,
    top_down::{actor::player::TakeAwayPlayerControl, Actor, ActorTarget},
};

const LETTERBOXING_FADE_IN_DURATION: Duration = from_millis(500);
const LETTERBOXING_FADE_OUT_DURATION: Duration = from_millis(250);
const LETTERBOXING_QUAD_FADE_IN_HEIGHT: Val = Val::Percent(15.0);

/// Will be true if there's a cutscene playing.
pub fn in_cutscene() -> impl FnMut(Option<Res<Cutscene>>) -> bool {
    move |cutscene| cutscene.is_some()
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
    /// Make sure you run the [`common_visuals::systems::interpolate`]
    /// system.
    ///
    /// <https://en.wikipedia.org/wiki/Letterboxing_(filming)>
    fn has_letterboxing() -> bool {
        false
    }

    /// Helper function that spawns the cutscene.
    /// You don't need to do anything beyond this.
    ///
    /// It's the caller's responsibility to ensure that another scene is not
    /// already running.
    fn spawn(self, cmd: &mut Commands)
    where
        Self: Sized,
    {
        spawn_cutscene(cmd, self);
    }
}

impl IntoCutscene for Vec<CutsceneStep> {
    fn sequence(self) -> Vec<CutsceneStep> {
        self
    }
}

/// There can be at most one cutscene running at a time.
/// Cutscene can be connected with a dialog though.
#[derive(Resource, Reflect)]
pub struct Cutscene {
    /// Cutscene is over and will be despawned soon.
    /// Sometimes, we want to have some outro animation playing, e.g. for
    /// letterboxing, and so we keep the [`Cutscene`] resource alive for a bit
    /// but this flag disables all logic.
    is_over: bool,
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

    /// Gets access to commands, can schedule some, and then immediately
    /// transitions to the next step.
    ScheduleCommands(fn(&mut Commands, &GlobalStore)),
    /// Simple await until this duration passes.
    Sleep(Duration),
    /// Inserts the [`TakeAwayPlayerControl`] component to the player entity.
    /// Ideal for the first step of the cutscene.
    TakeAwayPlayerControl(Entity),
    /// Removes the [`TakeAwayPlayerControl`] component from the player entity.
    /// This gives the player back control, ideal for the last step of the
    /// cutscene.
    ReturnPlayerControl(Entity),
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
    ///
    /// You might want to couple this with [`StartLoadingScreen`].
    ChangeGlobalState {
        /// The state to change to.
        /// Typically this would be a quitting state so that we transition into
        /// a new scene.
        to: GlobalGameState,
        /// The transition to use.
        /// Pair this with the `to` field to guide the game into the next
        /// scene.
        with: GlobalGameStateTransition,
    },
    /// Sets the loading screen state to start and inserts the given loading
    /// settings.
    ///
    /// # Important
    /// Depending on the scene system setup, you might need to first
    /// [`Self::ChangeGlobalState`] before starting the loading screen.
    /// Otherwise the current scene might think that it's the one being loaded.
    ///
    /// # Panics
    /// If the loading screen state is not ready to start.
    StartLoadingScreen {
        /// The loading screen settings to use.
        settings: Option<LoadingScreenSettings>,
    },
    /// Waits until loading screen is loaded.
    WaitForLoadingScreen,
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
    BeginPortraitDialog(dialog::DialogRef),
    /// Waits until there is no portrait dialog resource.
    WaitForPortraitDialogToEnd,
    /// Inserts [`common_visuals::TranslationInterpolation`] to the
    /// given entity. The entity must have [`Transform`] component and the
    /// [`common_visuals::systems::interpolate`] system must be run.
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
    /// Starts color interpolation on the given entity.
    BeginColorInterpolation {
        /// Which entity to color.
        /// Must work with [`BeginInterpolationEvent`].
        who: Entity,
        /// What is the target color.
        to: Color,
        /// How long should this translation take.
        over: Duration,
        /// By default linear interpolation is used.
        animation_curve: Option<CubicSegment<Vec2>>,
    },
    /// Sets the [`Actor::direction`] property to the given value.
    /// This will be overwritten if the actor starts walking.
    SetActorFacingDirection(Entity, GridDirection),
    /// Removes the [`SyncWithPlayer`] component from [`MainCamera`] entity and
    /// adds [`ManualControl`] component. This is useful for manual camera
    /// control.
    ///
    /// [`ManualControl`] must be returned with
    /// [`Self::ReleaseManualMainCameraControl`].
    ClaimManualMainCameraControl,
    /// Undoes [`Self::ClaimManualMainCameraControl`] by inserting
    /// [`ManualControl`] to the [`MainCamera`] entity.
    ReleaseManualMainCameraControl,
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
///
/// It's the caller's responsibility to ensure that another scene is not already
/// running.
pub fn spawn_cutscene<Scene: IntoCutscene>(
    cmd: &mut Commands,
    cutscene: Scene,
) {
    let sequence = cutscene.sequence();
    debug_assert!(!sequence.is_empty());
    let has_letterboxing = Scene::has_letterboxing();

    let letterboxing_entities = if has_letterboxing {
        // RenderLayers will be inserted once the cutscene is loaded.
        // This is to prevent a flicker when the quads are rendered but the
        // pixel zoom is not yet set.
        let camera = cmd
            .spawn((
                Name::new("Cutscene camera"),
                LetterboxingCamera,
                RenderLayers::layer(render_layer::CUTSCENE_LETTERBOXING),
                Camera2dBundle {
                    camera: Camera {
                        hdr: true,
                        order: order::CUTSCENE_LETTERBOXING,
                        clear_color: ClearColorConfig::None,
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        let mut top_entities = cmd.spawn((
            Name::new("Letterboxing: top quad"),
            LetterboxingTopQuad,
            TargetCamera(camera),
            RenderLayers::layer(render_layer::CUTSCENE_LETTERBOXING),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(0.0),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    top: Val::Percent(0.0),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
        ));
        let top = top_entities.id();
        BeginInterpolationEvent::of_ui_style_height(
            top,
            None,
            LETTERBOXING_QUAD_FADE_IN_HEIGHT,
        )
        .over(LETTERBOXING_FADE_IN_DURATION)
        .insert_to(&mut top_entities);

        let mut bottom_entities = cmd.spawn((
            Name::new("Letterboxing: bottom quad"),
            LetterboxingBottomQuad,
            TargetCamera(camera),
            RenderLayers::layer(render_layer::CUTSCENE_LETTERBOXING),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(0.0),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    bottom: Val::Percent(0.0),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
        ));
        let bottom = bottom_entities.id();
        BeginInterpolationEvent::of_ui_style_height(
            top,
            None,
            LETTERBOXING_QUAD_FADE_IN_HEIGHT,
        )
        .over(LETTERBOXING_FADE_IN_DURATION)
        .insert_to(&mut bottom_entities);

        Some([camera, top, bottom])
    } else {
        None
    };

    let cutscene = Cutscene {
        is_over: false,
        sequence,
        sequence_index: 0,
        stopwatch: Stopwatch::new(),
        letterboxing_entities,
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
) {
    if !cutscene.is_over {
        cutscene.stopwatch.tick(time.delta());
        cutscene.schedule_current_step(&mut cmd);
    }
}

impl Cutscene {
    fn schedule_next_step_or_despawn(&mut self, cmd: &mut Commands) {
        if self.is_over {
            return;
        }

        if self.sequence_index >= self.sequence.len() - 1 {
            self.force_despawn(cmd);
        } else {
            self.stopwatch.reset();
            self.sequence_index += 1;
            self.schedule_current_step(cmd);
        }
    }

    /// Despawns cutscene regardless of whether it finished.
    fn force_despawn(&mut self, cmd: &mut Commands) {
        self.is_over = true;

        if let Some(entities) = self.letterboxing_entities.take() {
            // smoothly animate quads out and when down, despawn everything

            BeginInterpolationEvent::of_ui_style_height(
                entities[2],
                None,
                Val::Percent(0.0),
            )
            .over(LETTERBOXING_FADE_OUT_DURATION)
            .insert(cmd);

            BeginInterpolationEvent::of_ui_style_height(
                entities[1],
                None,
                Val::Percent(0.0),
            )
            .over(LETTERBOXING_FADE_OUT_DURATION)
            .when_finished_do(move |cmd: &mut Commands| {
                trace!("Despawning cutscene");
                cmd.remove_resource::<Cutscene>();

                cmd.entity(entities[0]).despawn_recursive();
                cmd.entity(entities[1]).despawn_recursive();
                cmd.entity(entities[2]).despawn_recursive();
            })
            .insert(cmd);
        } else {
            cmd.remove_resource::<Cutscene>();
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
    take_away_player_control: SystemId,
    return_player_control: SystemId,
    sleep: SystemId,
    insert_atlas_animation_timer_to: SystemId,
    change_global_state: SystemId,
    start_loading_screen: SystemId,
    wait_for_loading_screen: SystemId,
    begin_simple_walk_to: SystemId,
    wait_until_actor_at_rest: SystemId,
    begin_portrait_dialog: SystemId,
    wait_for_portrait_dialog_to_end: SystemId,
    reverse_atlas_animation: SystemId,
    wait_until_atlas_animation_ends: SystemId,
    begin_moving_entity: SystemId,
    set_actor_facing_direction: SystemId,
    schedule_commands: SystemId,
    claim_manual_main_camera_control: SystemId,
    release_manual_main_camera_control: SystemId,
    begin_color_interpolation: SystemId,
}

impl CutsceneSystems {
    fn register_systems(w: &mut World) -> Self {
        Self {
            if_true_this_else_that: w.register_system(if_true_this_else_that),
            take_away_player_control: w
                .register_system(take_away_player_control),
            return_player_control: w.register_system(return_player_control),
            sleep: w.register_system(sleep),
            insert_atlas_animation_timer_to: w
                .register_system(insert_atlas_animation_timer_to),
            change_global_state: w.register_system(change_global_state),
            start_loading_screen: w.register_system(start_loading_screen),
            wait_for_loading_screen: w.register_system(wait_for_loading_screen),
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
            set_actor_facing_direction: w
                .register_system(set_actor_facing_direction),
            schedule_commands: w.register_system(schedule_commands),
            claim_manual_main_camera_control: w
                .register_system(claim_manual_main_camera_control),
            release_manual_main_camera_control: w
                .register_system(release_manual_main_camera_control),
            begin_color_interpolation: w
                .register_system(begin_color_interpolation),
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
        TakeAwayPlayerControl(_) => s.take_away_player_control,
        ReturnPlayerControl(_) => s.return_player_control,
        Sleep(_) => s.sleep,
        InsertAtlasAnimationTimerTo { .. } => s.insert_atlas_animation_timer_to,
        ChangeGlobalState { .. } => s.change_global_state,
        StartLoadingScreen { .. } => s.start_loading_screen,
        WaitForLoadingScreen => s.wait_for_loading_screen,
        BeginSimpleWalkTo { .. } => s.begin_simple_walk_to,
        WaitUntilActorAtRest(_) => s.wait_until_actor_at_rest,
        BeginPortraitDialog(_) => s.begin_portrait_dialog,
        WaitForPortraitDialogToEnd => s.wait_for_portrait_dialog_to_end,
        ReverseAtlasAnimation(_) => s.reverse_atlas_animation,
        WaitUntilAtlasAnimationEnds(_) => s.wait_until_atlas_animation_ends,
        BeginMovingEntity { .. } => s.begin_moving_entity,
        SetActorFacingDirection(_, _) => s.set_actor_facing_direction,
        ScheduleCommands(_) => s.schedule_commands,
        ClaimManualMainCameraControl => s.claim_manual_main_camera_control,
        BeginColorInterpolation { .. } => s.begin_color_interpolation,
        ReleaseManualMainCameraControl => s.release_manual_main_camera_control,
    }
}

fn take_away_player_control(mut cmd: Commands, mut cutscene: ResMut<Cutscene>) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::TakeAwayPlayerControl(entity) = &step else {
        panic!("Expected TakeAwayPlayerControl step, got {step}");
    };

    cmd.entity(*entity).insert(TakeAwayPlayerControl);

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn return_player_control(mut cmd: Commands, mut cutscene: ResMut<Cutscene>) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::ReturnPlayerControl(entity) = &step else {
        panic!("Expected ReturnPlayerControl step, got {step}");
    };

    cmd.entity(*entity).remove::<TakeAwayPlayerControl>();

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
    mut transition: ResMut<GlobalGameStateTransition>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::ChangeGlobalState { to, with } = &step else {
        panic!("Expected ChangeGlobalState step, got {step}");
    };

    next_state.set(*to);
    *transition = *with;

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn start_loading_screen(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
    existing_loading_screen_settings: Option<Res<LoadingScreenSettings>>,
    current_loading_screen_state: Res<State<LoadingScreenState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::StartLoadingScreen { settings } = &step else {
        panic!("Expected StartLoadingScreen step, got {step}");
    };

    if !current_loading_screen_state.is_ready_to_start() {
        panic!("Loading screen state is not ready to start");
    }
    next_loading_screen_state.set(common_loading_screen::start_state());

    if let Some(settings) = settings {
        if existing_loading_screen_settings.is_some() {
            error!("Overwriting loading screen settings");
        }
        cmd.insert_resource(settings.clone());
    };

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn wait_for_loading_screen(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
    current_loading_screen_state: Res<State<LoadingScreenState>>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::WaitForLoadingScreen = &step else {
        panic!("Expected WaitForLoadingScreen step, got {step}");
    };

    if current_loading_screen_state.is_waiting_for_signal() {
        cutscene.schedule_next_step_or_despawn(&mut cmd);
    }
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
        actor.walking_to = ActorTarget {
            square: *square,
            planned: *planned,
            since: Stopwatch::new(),
        }
        .into();
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
        if actor.walking_to.is_still() {
            cutscene.schedule_next_step_or_despawn(&mut cmd);
        }
    }
}

fn begin_portrait_dialog(mut cmd: Commands, mut cutscene: ResMut<Cutscene>) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::BeginPortraitDialog(dialog) = &step else {
        panic!("Expected BeginDialog step, got {step}");
    };

    cmd.insert_resource(
        StartDialogWhenLoaded::portrait().add_ref(dialog.clone()),
    );

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn wait_for_portrait_dialog_to_end(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
    dialog: Option<Res<PortraitDialog>>,
    loading: Option<Res<StartDialogWhenLoaded>>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::WaitForPortraitDialogToEnd = &step else {
        panic!("Expected WaitForPortraitDialogToEnd step, got {step}");
    };

    if dialog.is_none() && loading.is_none() {
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
        animation.play = match animation.play {
            AtlasAnimationStep::Forward => AtlasAnimationStep::Backward,
            AtlasAnimationStep::Backward => AtlasAnimationStep::Forward,
        };
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
            .insert(&mut cmd);
    }

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn set_actor_facing_direction(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,

    mut actors: Query<&mut Actor>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::SetActorFacingDirection(entity, direction) = &step else {
        panic!("Expected SetActorFacingDirection step, got {step}");
    };

    if let Ok(mut actor) = actors.get_mut(*entity) {
        actor.direction = *direction;
    }

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn schedule_commands(
    mut cmd: Commands,
    store: Res<GlobalStore>,
    mut cutscene: ResMut<Cutscene>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::ScheduleCommands(f) = &step else {
        panic!("Expected ScheduleCommands step, got {step}");
    };

    f(&mut cmd, &store);

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn claim_manual_main_camera_control(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,

    camera: Query<Entity, With<MainCamera>>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::ClaimManualMainCameraControl = &step else {
        panic!("Expected ClaimManualMainCameraControl step, got {step}");
    };

    if let Some(entity) = camera.get_single_or_none() {
        cmd.entity(entity)
            .remove::<SyncWithPlayer>()
            .insert(ManualControl);
    }

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn release_manual_main_camera_control(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,

    camera: Query<Entity, With<MainCamera>>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::ReleaseManualMainCameraControl = &step else {
        panic!("Expected ReleaseManualMainCameraControl step, got {step}");
    };

    if let Some(entity) = camera.get_single_or_none() {
        cmd.entity(entity).remove::<ManualControl>();
    }

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}

fn begin_color_interpolation(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::BeginColorInterpolation {
        who,
        to,
        over,
        animation_curve,
    } = &step
    else {
        panic!("Expected BeginColorInterpolation step, got {step}");
    };

    BeginInterpolationEvent::of_color(*who, None, *to)
        .over(*over)
        .with_animation_opt_curve(animation_curve.clone())
        .insert(&mut cmd);

    cutscene.schedule_next_step_or_despawn(&mut cmd);
}
