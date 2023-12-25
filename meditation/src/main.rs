//! Player controls weather sprite.
//!
//! The controls are WASD (or arrow keys) to move and move+space to activate
//! the special. The sprite should feel floaty as if you were playing Puff in
//! Smashbros.

#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod background;
mod climate;
mod control_mode;
mod distractions;
mod gravity;
mod path;
mod prelude;
mod ui;
mod weather;
mod zindex;

mod consts {
    /// What's shown on screen.
    pub(crate) const VISIBLE_WIDTH: f32 = 640.0;
    /// What's shown on screen.
    pub(crate) const VISIBLE_HEIGHT: f32 = 360.0;

    /// The stage is bigger than what's shown on screen.
    pub(crate) const GRAVITY_STAGE_WIDTH: f32 = VISIBLE_WIDTH * 1.25;

    /// The stage is bigger than what's shown on screen.
    pub(crate) const GRAVITY_STAGE_HEIGHT: f32 = VISIBLE_HEIGHT * 1.25;

    pub(crate) const PIXEL_ZOOM: f32 = 3.0;
}

use bevy::{
    render::{camera::RenderTarget, view::RenderLayers},
    window::WindowTheme,
};
use bevy_magic_light_2d::{
    gi::{
        compositing::{CameraTargets, PostProcessingMaterial},
        BevyMagicLight2DPlugin, LightScene,
    },
    MainCamera,
};
use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport, PixelZoom};
use prelude::*;

/// TODO: make it a resource
#[derive(Component)]
struct Game;

#[derive(Component)]
struct Paused;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::WARN,
                filter: "meditation=trace,meditation::weather::sprite=debug"
                    .to_string(),
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ciesin".into(),
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    mode: bevy::window::WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((
        BevyMagicLight2DPlugin,
        PixelCameraPlugin,
        bevy_webp_anim::Plugin,
        common_physics::Plugin,
        common_visuals::Plugin,
        ui::Plugin,
        climate::Plugin,
        distractions::Plugin,
        weather::Plugin,
    ))
    .insert_resource(ClearColor(Color::hex(background::COLOR).unwrap()))
    .insert_resource(gravity::field())
    .add_systems(Startup, (setup, background::spawn));

    BackgroundLightScene::init(&mut app);

    common_physics::poissons_equation::register::<gravity::Gravity>(&mut app);

    #[cfg(feature = "dev")]
    app.add_systems(Last, (path::visualize,));

    #[cfg(feature = "dev-poissons")]
    common_physics::poissons_equation::register_visualization::<
        gravity::Gravity,
        gravity::ChangeOfBasis,
        gravity::ChangeOfBasis,
    >(&mut app);

    app.run();
}

#[derive(Default, Clone, TypePath)]
struct BackgroundLightScene;

impl LightScene for BackgroundLightScene {
    fn render_layer_index() -> u8 {
        (RenderLayers::TOTAL_LAYERS - 1) as u8
    }

    fn post_processing_quad() -> Handle<Mesh> {
        Handle::weak_from_u128(23475629871623176235)
    }
    fn post_processing_material() -> Handle<PostProcessingMaterial<Self>> {
        Handle::weak_from_u128(52374048672736472871)
    }
    fn floor_image_handle() -> Handle<Image> {
        Handle::weak_from_u128(9127312736151891273)
    }
    fn sdf_target() -> Handle<Image> {
        Handle::weak_from_u128(2390847209461232343)
    }
    fn ss_probe_target() -> Handle<Image> {
        Handle::weak_from_u128(3423231236817235162)
    }
    fn ss_bounce_target() -> Handle<Image> {
        Handle::weak_from_u128(3198273198312367527)
    }
    fn ss_blend_target() -> Handle<Image> {
        Handle::weak_from_u128(7782312739182735881)
    }
    fn ss_filter_target() -> Handle<Image> {
        Handle::weak_from_u128(8761232615172413412)
    }
    fn ss_pose_target() -> Handle<Image> {
        Handle::weak_from_u128(4728165084756128470)
    }
}

fn setup(
    mut commands: Commands,
    camera_targets: Res<CameraTargets<BackgroundLightScene>>,
) {
    commands.spawn(Game);

    commands
        .spawn((
            PixelZoom::Fixed(consts::PIXEL_ZOOM as i32),
            PixelViewport,
            MainCamera,
        ))
        .insert(Camera2dBundle {
            camera: Camera {
                hdr: true,
                target: RenderTarget::Image(
                    camera_targets.floor_target.clone(),
                ),
                ..default()
            },
            projection: OrthographicProjection {
                near: -2000.0,
                ..default()
            },
            ..default()
        });
}
