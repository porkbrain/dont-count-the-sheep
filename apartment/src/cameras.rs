use bevy::render::view::RenderLayers;
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use main_game_lib::PIXEL_ZOOM;

use crate::prelude::*;

pub(crate) const BG_RENDER_LAYER: u8 = 2;
pub(crate) const CHARACTERS_RENDER_LAYER: u8 = 1;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn)
            .add_systems(OnEnter(GlobalGameState::ApartmentQuitting), despawn);
    }

    fn finish(&self, _: &mut App) {
        //
    }
}

#[derive(Component)]
struct CameraEntity;

fn spawn(mut commands: Commands) {
    commands.spawn((
        CameraEntity,
        PixelZoom::Fixed(PIXEL_ZOOM as i32),
        PixelViewport,
        RenderLayers::from_layers(&[
            0, // for FPS and other debug tools
            BG_RENDER_LAYER,
            CHARACTERS_RENDER_LAYER,
        ]),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: 1,
                ..default()
            },
            ..default()
        },
    ));
}

fn despawn(mut commands: Commands, bg: Query<Entity, With<CameraEntity>>) {
    for entity in bg.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
