use crate::cameras::BG_RENDER_LAYER;
use crate::prelude::*;
use bevy::render::view::RenderLayers;

#[derive(Component)]
struct BedroomEntity;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn);
        app.add_systems(OnEnter(GlobalGameState::ApartmentQuitting), despawn);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        BedroomEntity,
        RenderLayers::layer(BG_RENDER_LAYER),
        SpriteBundle {
            texture: asset_server.load(assets::BEDROOM_BG),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                zindex::BEDROOM_BG,
            )),
            ..default()
        },
    ));

    for (asset, zindex) in [
        (
            assets::BEDROOM_FURNITURE1,
            zindex::BEDROOM_FURNITURE_DISTANT,
        ),
        (assets::BEDROOM_FURNITURE2, zindex::BEDROOM_FURNITURE_MIDDLE),
        (
            assets::BEDROOM_FURNITURE3,
            zindex::BEDROOM_FURNITURE_CLOSEST,
        ),
    ] {
        commands.spawn((
            BedroomEntity,
            RenderLayers::layer(BG_RENDER_LAYER),
            SpriteBundle {
                texture: asset_server.load(asset),
                transform: Transform::from_translation(Vec3::new(
                    0.0, 0.0, zindex,
                )),
                ..default()
            },
        ));
    }
}

fn despawn(query: Query<Entity, With<BedroomEntity>>, mut commands: Commands) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
