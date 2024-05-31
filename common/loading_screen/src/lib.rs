//! A loading screen state machine.
//! The current system is quite tricky to integrate right and one has to combine
//! it with the main state machine in a specific way.
//!
//! The important state is [`LoadingScreenState::WaitForSignalToFinish`].
//! When the loading screen enters this state, it expects user call to the
//! [`finish`] function or change of the state to the [`finish_state`] state.
//! Once that's done, the loading screen will transition to opacity 0 and
//! then will despawn.
//!
//! Optionally, a background atlas can be chosen to be displayed during the
//! loading screen.

#![feature(trivial_bounds)]
#![deny(missing_docs)]

mod atlases;

use std::time::Duration;

pub use atlases::LoadingScreenAtlas;
use bevy::{
    math::vec3, prelude::*, render::view::RenderLayers, utils::Instant,
};
use common_visuals::{
    camera::{order, render_layer},
    PRIMARY_COLOR,
};

/// Slow fade in is the default, can be changed in [`LoadingScreenSettings`].
pub const DEFAULT_FADE_LOADING_SCREEN_IN: Duration = Duration::from_millis(400);
/// Fast fade out is the default, can be changed in [`LoadingScreenSettings`].
pub const DEFAULT_FADE_LOADING_SCREEN_OUT: Duration =
    Duration::from_millis(100);
/// How many times to scale the original loading image.
pub const LOADING_IMAGE_TRANSFORM_SCALE: f32 = 5.0;

/// A state machine where the states are the steps of the loading screen.
/// They are executed in order and loop back to the beginning.
///
/// Not recommended to use directly as there are race conditions involved if not
/// handled properly.
/// Use provided transition systems to change the state:
/// - [`start_state`] to begin the loading screen process
/// - [`finish_state`] to finish the loading screen process
#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Reflect)]
pub enum LoadingScreenState {
    /// This is the state in which the loading screen is not active and waiting
    /// to be activated.
    /// Change the state to [`start_state`] to activate the loading screen.
    #[default]
    DoNothing,
    /// 1. Spawn a camera with highest order.
    /// 2. Spawn a quad of bg color across the whole screen with opacity 0.
    /// 3. Spawn an image with visibility none.
    SpawnLoadingScreen,
    /// 4.
    FadeInQuadWhileBgLoading,
    /// 5. Wait
    /// 6. Set visibility of the image to visible (if no bg image go to
    ///    [`LoadingScreenState::StareAtLoadingScreen`])
    WaitForAtlasToLoad,
    /// 7. Fades out and sets the state to
    ///    [`LoadingScreenState::StareAtLoadingScreen`]. (skipped if no bg
    ///    image)
    FadeOutQuadToShowAtlas,
    /// 8. If requested, stay on this screen for given amount of time before
    ///    transitioning to [`LoadingScreenState::WaitForSignalToFinish`].
    StareAtLoadingScreen,
    /// 9. Now we wait for the loading to be done, user must [`finish_state`].
    WaitForSignalToFinish,
    /// 10. Fade in (if no bg image go to
    ///     [`LoadingScreenState::FadeOutQuadToShowGame`])
    FadeInQuadToRemoveAtlas,
    /// 11.
    /// (skipped if no bg image)
    RemoveAtlas,
    /// 12.
    FadeOutQuadToShowGame,
    /// 13.
    DespawnLoadingScreen,
}

/// Settings for the loading screen state machine.
/// We implement default for this which should be used because the settings will
/// expand.
#[derive(Resource, Debug, Reflect, Clone)]
pub struct LoadingScreenSettings {
    /// If set to none:
    /// - [`LoadingScreenState::WaitForAtlasToLoad`] goes straight to
    ///   [`LoadingScreenState::WaitForSignalToFinish`]
    /// - [`LoadingScreenState::FadeInQuadToRemoveAtlas`] goes straight to
    ///   [`LoadingScreenState::FadeOutQuadToShowGame`]
    pub atlas: Option<LoadingScreenAtlas>,
    /// How long does it take to fade in the quad that hides the load out
    /// scene.
    pub fade_loading_screen_in: Duration,
    /// How long does it take to fade out the quad that reveals the load in
    /// scene.
    pub fade_loading_screen_out: Duration,
    /// If bg image not present, this value is ignored.
    pub stare_at_loading_screen_for_at_least: Option<Duration>,
}

/// Set the state to this to open loading screen.
/// You must ensure that the state is [`LoadingScreenState::DoNothing`],
/// other you break stuff.
/// You must insert [`LoadingScreenSettings`] before you set the state.
pub fn start_state() -> LoadingScreenState {
    LoadingScreenState::SpawnLoadingScreen
}

/// When you are done loading, set the state to this to smoothly hide the
/// loading screen. Your game camera should be ready to show the game.
/// Make sure to call this only if the current state is
/// [`LoadingScreenState::WaitForSignalToFinish`].
pub fn finish_state() -> LoadingScreenState {
    LoadingScreenState::FadeInQuadToRemoveAtlas
}

/// In this state the loading screen is waiting for the user to set it to the
/// state [`finish_state`] or call the [`finish`] system.
pub fn wait_state() -> LoadingScreenState {
    LoadingScreenState::WaitForSignalToFinish
}

/// Sets the state to [`finish_state`].
pub fn finish(mut next_state: ResMut<NextState<LoadingScreenState>>) {
    next_state.set(finish_state());
}

/// Adds the state machine to the app.
/// Doesn't do anything until the state is set to [`start_state`].
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LoadingScreenState>()
            .register_type::<LoadingScreenState>();

        app.add_systems(
            OnEnter(LoadingScreenState::SpawnLoadingScreen),
            spawn_loading_screen,
        )
        .add_systems(
            Update,
            fade_in_quad_while_bg_loading
                .run_if(in_state(LoadingScreenState::FadeInQuadWhileBgLoading)),
        )
        .add_systems(
            Update,
            stare_at_loading_screen
                .run_if(in_state(LoadingScreenState::StareAtLoadingScreen)),
        )
        .add_systems(
            Update,
            wait_for_bg_to_load
                .run_if(in_state(LoadingScreenState::WaitForAtlasToLoad)),
        )
        .add_systems(
            Update,
            fade_out_quad_to_show_atlas
                .run_if(in_state(LoadingScreenState::FadeOutQuadToShowAtlas)),
        )
        .add_systems(
            Update,
            fade_in_quad_that_hides_atlas
                .run_if(in_state(LoadingScreenState::FadeInQuadToRemoveAtlas)),
        )
        .add_systems(OnEnter(LoadingScreenState::RemoveAtlas), remove_bg)
        .add_systems(
            Update,
            fade_out_quad_to_show_game
                .run_if(in_state(LoadingScreenState::FadeOutQuadToShowGame)),
        )
        .add_systems(
            OnEnter(LoadingScreenState::DespawnLoadingScreen),
            despawn_loading_screen,
        );

        #[cfg(feature = "devtools")]
        {
            app.register_type::<LoadingScreenState>()
                .register_type::<LoadingScreenSettings>();

            use bevy_inspector_egui::quick::{
                ResourceInspectorPlugin, StateInspectorPlugin,
            };
            app.add_plugins((
                StateInspectorPlugin::<LoadingScreenState>::default(),
                ResourceInspectorPlugin::<LoadingScreenSettings>::default(),
            ));
        }
    }
}

#[derive(Component)]
struct LoadingCamera;

#[derive(Component)]
struct LoadingImage;

#[derive(Component)]
struct LoadingQuad;

fn spawn_loading_screen(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<LoadingScreenSettings>,
    mut next_state: ResMut<NextState<LoadingScreenState>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    trace!("Spawning loading screen");

    let camera = cmd
        .spawn((
            Name::from("Loading screen camera"),
            LoadingCamera,
            RenderLayers::layer(render_layer::LOADING),
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    order: order::LOADING,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    // quad
    cmd.spawn((
        Name::from("Loading screen quad"),
        LoadingQuad,
        RenderLayers::layer(render_layer::LOADING),
        TargetCamera(camera),
        NodeBundle {
            background_color: BackgroundColor({
                let mut c = PRIMARY_COLOR;
                c.set_a(0.0);
                c
            }),
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            // in front of the image
            z_index: ZIndex::Global(1),
            ..default()
        },
    ));

    // bg image
    if let Some(atlas) = settings.atlas {
        let texture: Handle<Image> = asset_server.load(atlas.asset_path());

        cmd.spawn(Name::new("Loading screen atlas bg"))
            .insert((
                LoadingImage,
                // we use the same handle to check whether the image is loaded,
                // but the actual image is rendered in the child entity
                texture.clone(),
                TargetCamera(camera),
                RenderLayers::layer(render_layer::LOADING),
            ))
            .insert(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                visibility: Visibility::Hidden,
                background_color: BackgroundColor(PRIMARY_COLOR),
                ..default()
            })
            .with_children(|parent| {
                let (layout, animation, timer) = atlas.thingies();

                parent
                    .spawn(Name::new("Loading screen atlas image"))
                    .insert((
                        animation,
                        timer,
                        RenderLayers::layer(render_layer::LOADING),
                    ))
                    .insert(AtlasImageBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Auto,
                            height: Val::Auto,
                            align_self: AlignSelf::Center,
                            margin: UiRect {
                                left: Val::Auto,
                                right: Val::Auto,
                                ..default()
                            },
                            ..default()
                        },
                        texture_atlas: TextureAtlas {
                            layout: atlas_layouts.add(layout),
                            ..default()
                        },
                        image: UiImage::new(
                            asset_server.load(atlas.asset_path()),
                        ),
                        transform: Transform::from_scale(vec3(
                            LOADING_IMAGE_TRANSFORM_SCALE,
                            LOADING_IMAGE_TRANSFORM_SCALE,
                            1.0,
                        )),
                        ..default()
                    });
            });
    }

    trace!("Loading screen spawned, entering next state");
    next_state.set(LoadingScreenState::FadeInQuadWhileBgLoading);
}

fn fade_in_quad_while_bg_loading(
    time: Res<Time>,
    next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    query: Query<&mut BackgroundColor, With<LoadingQuad>>,
) {
    fade_quad(
        Fade::In,
        settings.fade_loading_screen_in,
        LoadingScreenState::WaitForAtlasToLoad,
        time,
        next_state,
        query,
    )
}

fn wait_for_bg_to_load(
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    mut image: Query<(&Handle<Image>, &mut Visibility), With<LoadingImage>>,
) {
    if settings.atlas.is_none() {
        next_state.set(LoadingScreenState::StareAtLoadingScreen);
        return;
    }

    let (image, mut visibility) = image.single_mut();

    if !asset_server.is_loaded_with_dependencies(image) {
        return;
    }

    trace!("Bg loaded");

    *visibility = Visibility::Visible;

    next_state.set(LoadingScreenState::FadeOutQuadToShowAtlas);
}

fn fade_out_quad_to_show_atlas(
    time: Res<Time>,
    next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    query: Query<&mut BackgroundColor, With<LoadingQuad>>,
) {
    fade_quad(
        Fade::Out,
        settings.fade_loading_screen_in, // symmetrical
        LoadingScreenState::StareAtLoadingScreen,
        time,
        next_state,
        query,
    )
}

fn stare_at_loading_screen(
    mut next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    mut since: Local<Option<Instant>>,
) {
    if let Some(min) = settings.stare_at_loading_screen_for_at_least {
        let elapsed = since.get_or_insert_with(Instant::now).elapsed();
        if min > elapsed {
            return;
        }
    }

    // reset local state for next time
    *since = None;

    // now we wait for the user to call change from state
    trace!("Waiting for a signal to finish loading screen");
    next_state.set(LoadingScreenState::WaitForSignalToFinish);
}

fn fade_in_quad_that_hides_atlas(
    time: Res<Time>,
    mut next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    query: Query<&mut BackgroundColor, With<LoadingQuad>>,
) {
    trace!("Received signal to finish the loading screen");

    if settings.atlas.is_none() {
        next_state.set(LoadingScreenState::FadeOutQuadToShowGame);
        return;
    }

    fade_quad(
        Fade::In,
        settings.fade_loading_screen_in, // symmetrical
        LoadingScreenState::RemoveAtlas,
        time,
        next_state,
        query,
    )
}

fn remove_bg(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<LoadingScreenState>>,

    mut query: Query<Entity, With<LoadingImage>>,
) {
    trace!("Removing bg");

    let entity = query.single_mut();

    cmd.entity(entity).despawn_recursive();

    next_state.set(LoadingScreenState::FadeOutQuadToShowGame);
}

fn fade_out_quad_to_show_game(
    time: Res<Time>,
    next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    query: Query<&mut BackgroundColor, With<LoadingQuad>>,
) {
    fade_quad(
        Fade::Out,
        settings.fade_loading_screen_out,
        LoadingScreenState::DespawnLoadingScreen,
        time,
        next_state,
        query,
    )
}

fn despawn_loading_screen(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<LoadingScreenState>>,

    camera: Query<Entity, (With<LoadingCamera>, Without<LoadingQuad>)>,
    quad: Query<Entity, (Without<LoadingCamera>, With<LoadingQuad>)>,
) {
    trace!("Despawning loading screen");

    cmd.remove_resource::<LoadingScreenSettings>();
    cmd.entity(camera.single()).despawn_recursive();
    cmd.entity(quad.single()).despawn_recursive();

    next_state.set(LoadingScreenState::DoNothing);
}

#[derive(Debug)]
enum Fade {
    In,
    Out,
}

fn fade_quad(
    fade: Fade,
    fade_duration: Duration,
    state_once_done: LoadingScreenState,

    time: Res<Time>,
    mut next_state: ResMut<NextState<LoadingScreenState>>,

    mut query: Query<&mut BackgroundColor, With<LoadingQuad>>,
) {
    let mut bg_color = query.single_mut();
    let BackgroundColor(ref mut color) = bg_color.as_mut();

    let alpha = color.a();
    let dt = time.delta_seconds() / fade_duration.as_secs_f32();

    let new_alpha = match fade {
        Fade::Out => alpha - dt,
        Fade::In => alpha + dt,
    };

    color.set_a(new_alpha.clamp(0.0, 1.0));

    if !(0.0..=1.0).contains(&new_alpha) {
        trace!("Done quad {fade:?}, next {state_once_done:?}");
        next_state.set(state_once_done);
    }
}

impl Default for LoadingScreenSettings {
    fn default() -> Self {
        Self {
            atlas: None,
            fade_loading_screen_in: DEFAULT_FADE_LOADING_SCREEN_IN,
            fade_loading_screen_out: DEFAULT_FADE_LOADING_SCREEN_OUT,
            stare_at_loading_screen_for_at_least: None,
        }
    }
}

impl LoadingScreenState {
    /// Returns true if the loading screen is ready to start.
    pub fn is_ready_to_start(self) -> bool {
        matches!(self, LoadingScreenState::DoNothing)
    }

    /// Returns true if the loading screen is waiting for the user to advance
    /// it.
    pub fn is_waiting_for_signal(self) -> bool {
        matches!(self, LoadingScreenState::WaitForSignalToFinish)
    }
}
