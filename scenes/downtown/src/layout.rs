use bevy::render::view::RenderLayers;
use common_visuals::camera::render_layer;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::prelude::*;

/// We arbitrarily derive the [`Default`] to allow reflection.
/// It does not have a meaningful default value.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    EnumIter,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Reflect,
    Serialize,
    strum::Display,
)]
#[reflect(Default)]
#[allow(clippy::enum_variant_names)]
pub enum DowntownTileKind {
    #[default]
    PlayerHouseEntrance,
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

impl common_top_down::layout::Tile for DowntownTileKind {
    #[inline]
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    #[inline]
    fn is_zone(&self) -> bool {
        match self {
            Self::PlayerHouseEntrance => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}
