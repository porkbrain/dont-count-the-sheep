use bevy::{
    core_pipeline::clear_color::ClearColorConfig, render::view::RenderLayers,
};
use bevy_pixel_camera::{PixelViewport, PixelZoom};

use crate::{prelude::*, PIXEL_ZOOM, VISIBLE_HEIGHT, VISIBLE_WIDTH};

pub const DEFAULT_FADE_LOADING_SCREEN_IN: Duration = from_millis(500);
pub const DEFAULT_FADE_LOADING_SCREEN_OUT: Duration = from_millis(100);

/// Dedicated for loading screen.
const LOADING_SCREEN_LAYER: u8 = 29;
/// Higher than any other.
const LOADING_SCREEN_ORDER: isize = 10;

/// A state machine where the states are the steps of the loading screen.
/// They are executed in order and loop back to the beginning.
///
/// Not recommended to use directly as there are race conditions involved if not
/// handled properly.
/// Use provided transition systems to change the state:
/// - [`start`] to begin the loading screen process
/// - [`finish`] to finish the loading screen process
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
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
    WaitForBgToLoad,
    /// 7. Fades out and sets the state to [`DoNothing`].
    FadeOutQuadToShowBg,
    /// 8. Now we wait for the loading to be done, user must call [`finish`].
    WaitForSignalToFinish,
    /// 9. Fade in
    FadeInQuadToHideBg,
    /// 10.
    RemoveBg,
    /// 11.
    FadeOutQuadToShowGame,
    /// 12.
    DespawnLoadingScreen,
}

#[derive(Resource)]
pub struct LoadingScreenSettings {
    pub bg_image_asset: Option<&'static str>,
    pub fade_loading_screen_in: Duration,
    pub fade_loading_screen_out: Duration,
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

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_state::<LoadingScreenState>();

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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<LoadingScreenSettings>,
    mut next_state: ResMut<NextState<LoadingScreenState>>,
) {
    trace!("Spawning loading screen");

    commands.spawn((
        LoadingCamera,
        PixelZoom::Fixed(PIXEL_ZOOM as i32),
        PixelViewport,
        RenderLayers::layer(LOADING_SCREEN_LAYER),
        Camera2dBundle {
            camera: Camera {
                order: LOADING_SCREEN_ORDER,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
    ));

    // quad
    commands.spawn((
        LoadingQuad,
        RenderLayers::layer(LOADING_SCREEN_LAYER),
        SpriteBundle {
            sprite: Sprite {
                color: {
                    let mut c = PRIMARY_COLOR;
                    c.set_a(0.0);
                    c
                },
                custom_size: Some(Vec2::new(VISIBLE_WIDTH, VISIBLE_HEIGHT)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                0.0, 0.0, 1.0, // in front of bg
            )),
            ..default()
        },
    ));

    // bg image
    commands.spawn((
        LoadingImage,
        RenderLayers::layer(LOADING_SCREEN_LAYER),
        SpriteBundle {
            texture: settings
                .bg_image_asset
                .map(|a| asset_server.load(a))
                .unwrap_or_default(),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));

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

    mut image: Query<(&Handle<Image>, &mut Visibility), With<LoadingImage>>,
) {
    let (image, mut visibility) = image.single_mut();

    if !asset_server.is_loaded_with_dependencies(image) {
        return;
    }

    *visibility = Visibility::Visible;

    trace!("Bg loaded");
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
        // now we wait for the user to call [`finish`]
        LoadingScreenState::WaitForSignalToFinish,
        time,
        next_state,
        query,
    )
}

fn fade_in_quad_to_hide_bg(
    time: Res<Time>,
    next_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    query: Query<&mut Sprite, With<LoadingQuad>>,
) {
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
    mut commands: Commands,
    mut next_state: ResMut<NextState<LoadingScreenState>>,

    mut query: Query<Entity, With<LoadingImage>>,
) {
    trace!("Removing bg");

    let entity = query.single_mut();

    commands.entity(entity).despawn_recursive();

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
    mut commands: Commands,
    mut next_state: ResMut<NextState<LoadingScreenState>>,

    camera: Query<Entity, (With<LoadingCamera>, Without<LoadingQuad>)>,
    quad: Query<Entity, (Without<LoadingCamera>, With<LoadingQuad>)>,
) {
    trace!("Despawning loading screen");

    commands.remove_resource::<LoadingScreenSettings>();
    commands.entity(camera.single()).despawn_recursive();
    commands.entity(quad.single()).despawn_recursive();

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

    if new_alpha > 1.0 || new_alpha < 0.0 {
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
        }
    }
}
