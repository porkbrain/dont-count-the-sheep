#![feature(trivial_bounds)]

use std::time::Duration;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*,
    render::view::RenderLayers, utils::Instant,
};
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use common_visuals::{
    camera::{
        order, render_layer, PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH,
        PIXEL_ZOOM,
    },
    PRIMARY_COLOR,
};

pub const DEFAULT_FADE_LOADING_SCREEN_IN: Duration = Duration::from_millis(500);
pub const DEFAULT_FADE_LOADING_SCREEN_OUT: Duration =
    Duration::from_millis(100);

/// A state machine where the states are the steps of the loading screen.
/// They are executed in order and loop back to the beginning.
///
/// Not recommended to use directly as there are race conditions involved if not
/// handled properly.
/// Use provided transition systems to change the state:
/// - [`start_state`] to begin the loading screen process
/// - [`finish_state`] to finish the loading screen process
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum LoadingScreenState {
    #[default]
    DoNothing,
    /// 1. Spawn a camera with highest order.
    /// 2. Spawn a quad of bg color across the whole screen with opacity 0.
    /// 3. Spawn an image with visibility none.
    SpawnLoadingScreen,
    /// 4.
    FadeInQuadWhileBgLoading,
    /// 5. Wait
    /// 6. Set visibility of the image to visible
    /// (if no bg image go to [`LoadingScreenState::StareAtLoadingScreen`])
    WaitForBgToLoad,
    /// 7. Fades out and sets the state to
    ///    [`LoadingScreenState::StareAtLoadingScreen`].
    /// (skipped if no bg image)
    FadeOutQuadToShowBg,
    /// 8. If requested, stay on this screen for given amount of time before
    ///    transitioning to [`LoadingScreenState::WaitForSignalToFinish`].
    StareAtLoadingScreen,
    /// 9. Now we wait for the loading to be done, user must [`finish_state`].
    WaitForSignalToFinish,
    /// 10. Fade in
    /// (if no bg image go to [`LoadingScreenState::FadeOutQuadToShowGame`])
    FadeInQuadToHideBg,
    /// 11.
    /// (skipped if no bg image)
    RemoveBg,
    /// 12.
    FadeOutQuadToShowGame,
    /// 13.
    DespawnLoadingScreen,
}

#[derive(Resource, Reflect)]
pub struct LoadingScreenSettings {
    /// If set to none:
    /// - [`LoadingScreenState::WaitForBgToLoad`] goes straight to
    ///   [`LoadingScreenState::WaitForSignalToFinish`]
    /// - [`LoadingScreenState::FadeInQuadToHideBg`] goes straight to
    ///   [`LoadingScreenState::FadeOutQuadToShowGame`]
    pub bg_image_asset: Option<&'static str>,
    pub fade_loading_screen_in: Duration,
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
    LoadingScreenState::FadeInQuadToHideBg
}
pub fn finish(mut next_state: ResMut<NextState<LoadingScreenState>>) {
    next_state.set(finish_state());
}

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_state::<LoadingScreenState>()
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
                .run_if(in_state(LoadingScreenState::WaitForBgToLoad)),
        )
        .add_systems(
            Update,
            fade_out_quad_to_show_bg
                .run_if(in_state(LoadingScreenState::FadeOutQuadToShowBg)),
        )
        .add_systems(
            Update,
            fade_in_quad_to_hide_bg
                .run_if(in_state(LoadingScreenState::FadeInQuadToHideBg)),
        )
        .add_systems(OnEnter(LoadingScreenState::RemoveBg), remove_bg)
        .add_systems(
            Update,
            fade_out_quad_to_show_game
                .run_if(in_state(LoadingScreenState::FadeOutQuadToShowGame)),
        )
        .add_systems(
            OnEnter(LoadingScreenState::DespawnLoadingScreen),
            despawn_loading_screen,
        );
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
) {
    trace!("Spawning loading screen");

    cmd.spawn((
        Name::from("Loading screen camera"),
        LoadingCamera,
        PixelZoom::Fixed(PIXEL_ZOOM),
        PixelViewport,
        RenderLayers::layer(render_layer::LOADING),
        UiCameraConfig { show_ui: false },
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: order::LOADING,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
    ));

    // quad
    cmd.spawn((
        Name::from("Loading screen quad"),
        LoadingQuad,
        RenderLayers::layer(render_layer::LOADING),
        SpriteBundle {
            sprite: Sprite {
                color: {
                    let mut c = PRIMARY_COLOR;
                    c.set_a(0.0);
                    c
                },
                custom_size: Some(Vec2::new(
                    PIXEL_VISIBLE_WIDTH,
                    PIXEL_VISIBLE_HEIGHT,
                )),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                0.0, 0.0, 1.0, // in front of bg
            )),
            ..default()
        },
    ));

    // bg image
    if let Some(bg_image_asset) = settings.bg_image_asset {
        cmd.spawn((
            Name::from("Loading screen image"),
            LoadingImage,
            RenderLayers::layer(render_layer::LOADING),
            SpriteBundle {
                texture: asset_server.load(bg_image_asset),
                visibility: Visibility::Hidden,
                ..default()
            },
        ));
    }

    next_state.set(LoadingScreenState::FadeInQuadWhileBgLoading);
}

fn fade_in_quad_while_bg_loading(
    time: Res<Time>,
    next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    query: Query<&mut Sprite, With<LoadingQuad>>,
) {
    fade_quad(
        Fade::In,
        settings.fade_loading_screen_in,
        LoadingScreenState::WaitForBgToLoad,
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
    if settings.bg_image_asset.is_none() {
        next_state.set(LoadingScreenState::StareAtLoadingScreen);
        return;
    }

    let (image, mut visibility) = image.single_mut();

    if !asset_server.is_loaded_with_dependencies(image) {
        return;
    }

    trace!("Bg loaded");

    *visibility = Visibility::Visible;

    next_state.set(LoadingScreenState::FadeOutQuadToShowBg);
}

fn fade_out_quad_to_show_bg(
    time: Res<Time>,
    next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    query: Query<&mut Sprite, With<LoadingQuad>>,
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
    next_state.set(LoadingScreenState::WaitForSignalToFinish);
}

fn fade_in_quad_to_hide_bg(
    time: Res<Time>,
    mut next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    query: Query<&mut Sprite, With<LoadingQuad>>,
) {
    if settings.bg_image_asset.is_none() {
        next_state.set(LoadingScreenState::FadeOutQuadToShowGame);
        return;
    }

    fade_quad(
        Fade::In,
        settings.fade_loading_screen_in, // symmetrical
        LoadingScreenState::RemoveBg,
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

    query: Query<&mut Sprite, With<LoadingQuad>>,
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

    mut query: Query<&mut Sprite, With<LoadingQuad>>,
) {
    let mut sprite = query.single_mut();

    let alpha = sprite.color.a();
    let dt = time.delta_seconds() / fade_duration.as_secs_f32();

    let new_alpha = match fade {
        Fade::Out => alpha - dt,
        Fade::In => alpha + dt,
    };

    sprite.color.set_a(new_alpha.clamp(0.0, 1.0));

    if !(0.0..=1.0).contains(&new_alpha) {
        trace!("Done quad {fade:?}, next {state_once_done:?}");
        next_state.set(state_once_done);
    }
}

impl Default for LoadingScreenSettings {
    fn default() -> Self {
        Self {
            bg_image_asset: None,
            fade_loading_screen_in: DEFAULT_FADE_LOADING_SCREEN_IN,
            fade_loading_screen_out: DEFAULT_FADE_LOADING_SCREEN_OUT,
            stare_at_loading_screen_for_at_least: None,
        }
    }
}
