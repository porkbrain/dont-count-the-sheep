use bevy::render::view::RenderLayers;
use bevy_grid_squared::SquareLayout;
use common_visuals::camera::render_layer;
use lazy_static::lazy_static;
use main_game_lib::{common_top_down::IntoMap, vec2_ext::Vec2Ext};

use crate::{prelude::*, Downtown};

lazy_static! {
    static ref LAYOUT: SquareLayout = SquareLayout {
        square_size: 6.0,
        origin: vec2(356.0, 175.0).as_top_left_into_centered(),
    };
}

pub(crate) mod zones {
    //
}

#[derive(Component)]
struct LayoutEntity;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::DowntownLoading), spawn);
        app.add_systems(OnExit(GlobalGameState::DowntownQuitting), despawn);
    }
}

fn spawn(mut cmd: Commands, asset_server: Res<AssetServer>) {
    #[allow(clippy::single_element_loop)]
    for (name, asset, zindex) in [("Background", assets::BG, zindex::BG)] {
        cmd.spawn((
            Name::from(name),
            LayoutEntity,
            RenderLayers::layer(render_layer::BG),
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

fn despawn(mut cmd: Commands, query: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

impl IntoMap for Downtown {
    type LocalTileKind = ();

    fn name() -> &'static str {
        "downtown"
    }

    fn bounds() -> [i32; 4] {
        [-80, 60, -20, 160]
    }

    fn asset_path() -> &'static str {
        assets::MAP
    }

    fn layout() -> &'static SquareLayout {
        &LAYOUT
    }
}
