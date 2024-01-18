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

use bevy::{ecs::system::SystemId, prelude::*, time::Stopwatch};
use bevy_grid_squared::{GridDirection, Square};
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_store::GlobalStore;
use common_story::portrait_dialog::{DialogRoot, PortraitDialog};
use common_top_down::{Actor, ActorTarget, Player};
use common_visuals::AnimationTimer;

use crate::{
    GlobalGameState, GlobalGameStateTransition, GlobalGameStateTransitionStack,
};

pub trait IntoCutscene {
    fn sequence(self) -> Vec<CutsceneStep>;

    /// If true, the cutscene will be letterboxed with two black bars: above and
    /// below the screen.
    /// A camera and two quads are spawned for this purpose.
    /// They are despawned when the cutscene ends.
    ///
    /// <https://en.wikipedia.org/wiki/Letterboxing_(filming)>
    fn letterboxing() -> bool {
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
    /// Whether there's letterboxing quads and camera spawned.
    /// If yes then they must be despawned when the cutscene ends.
    has_letterboxing: bool,
}

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
    /// Inserts [`AnimationTimer`] component to the given entity.
    InsertAnimationTimerTo {
        entity: Entity,
        duration: Duration,
        mode: TimerMode,
    },
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
}

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let w = &mut app.world;

        let systems = CutsceneSystems::register_systems(w);

        CUTSCENE_SYSTEMS.get_or_init(|| systems);

        app.add_systems(First, schedule_current_step.run_if(in_cutscene()));
    }
}

/// Will be true if there's a cutscene playing.
pub fn in_cutscene() -> impl FnMut(Option<Res<Cutscene>>) -> bool {
    move |cutscene| cutscene.is_some()
}

pub fn spawn_cutscene<Scene: IntoCutscene>(
    cmd: &mut Commands,
    cutscene: Scene,
) {
    let sequence = cutscene.sequence();
    debug_assert!(!sequence.is_empty());

    let cutscene = Cutscene {
        sequence,
        sequence_index: 0,
        stopwatch: Stopwatch::new(),
        has_letterboxing: Scene::letterboxing(),
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
    cutscene.stopwatch.tick(time.delta());
    cutscene.schedule_current_step(&mut cmd);
}

impl Cutscene {
    fn schedule_next_step_or_despawn(&mut self, cmd: &mut Commands) {
        if self.sequence_index >= self.sequence.len() - 1 {
            cmd.remove_resource::<Cutscene>();

            if self.has_letterboxing {
                // TODO: despawn letterboxing quads and camera
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
    insert_animation_timer_to: SystemId,
    change_global_state: SystemId,
    begin_simple_walk_to: SystemId,
    wait_until_actor_at_rest: SystemId,
    begin_portrait_dialog: SystemId,
    wait_for_portrait_dialog_to_end: SystemId,
}

impl CutsceneSystems {
    fn register_systems(w: &mut World) -> Self {
        Self {
            if_true_this_else_that: w.register_system(if_true_this_else_that),
            remove_player_component: w.register_system(remove_player_component),
            add_player_component: w.register_system(add_player_component),
            sleep: w.register_system(sleep),
            insert_animation_timer_to: w
                .register_system(insert_animation_timer_to),
            change_global_state: w.register_system(change_global_state),
            begin_simple_walk_to: w.register_system(begin_simple_walk_to),
            wait_until_actor_at_rest: w
                .register_system(wait_until_actor_at_rest),
            begin_portrait_dialog: w.register_system(begin_portrait_dialog),
            wait_for_portrait_dialog_to_end: w
                .register_system(wait_for_portrait_dialog_to_end),
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
        InsertAnimationTimerTo { .. } => s.insert_animation_timer_to,
        ChangeGlobalState { .. } => s.change_global_state,
        BeginSimpleWalkTo { .. } => s.begin_simple_walk_to,
        WaitUntilActorAtRest(_) => s.wait_until_actor_at_rest,
        BeginPortraitDialog(_) => s.begin_portrait_dialog,
        WaitForPortraitDialogToEnd => s.wait_for_portrait_dialog_to_end,
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

fn insert_animation_timer_to(
    mut cmd: Commands,
    mut cutscene: ResMut<Cutscene>,
) {
    let step = &cutscene.sequence[cutscene.sequence_index];
    let CutsceneStep::InsertAnimationTimerTo {
        entity,
        duration,
        mode,
    } = &step
    else {
        panic!("Expected InsertAnimationTimerTo step, got {step}");
    };

    cmd.entity(*entity)
        .insert(AnimationTimer::new(*duration, *mode));
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
            return;
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

    let sequence = if cond(&*store) {
        *on_true.clone()
    } else {
        *on_false.clone()
    };

    cutscene.sequence_index = 0;
    cutscene.sequence = sequence;

    cutscene.schedule_current_step(&mut cmd);
}
