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

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
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

    commands.spawn((
        BedroomEntity,
        RenderLayers::layer(BG_RENDER_LAYER),
        SpriteBundle {
            texture: asset_server.load(assets::BEDROOM_FURNITURE),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                zindex::BEDROOM_FURNITURE,
            )),
            ..default()
        },
    ));

    next_state.set(GlobalGameState::InApartment);
}

fn despawn(query: Query<Entity, With<BedroomEntity>>, mut commands: Commands) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
