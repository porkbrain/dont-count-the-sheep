use bevy::render::view::RenderLayers;
use common_rscn::{NodeName, TscnSpawner, TscnTree, TscnTreeHandle};
use common_store::GlobalStore;
use common_top_down::TileMap;
use common_visuals::camera::render_layer;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{actor::DowntownAction, prelude::*, Downtown};

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
    strum::EnumString,
)]
#[reflect(Default)]
#[allow(clippy::enum_variant_names)]
pub enum DowntownTileKind {
    #[default]
    PlayerHouseEntrance,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "devtools")]
        app.register_type::<DowntownTileKind>();

        app.add_systems(
            OnEnter(GlobalGameState::DowntownLoading),
            common_rscn::start_loading_tscn::<Downtown>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_state(GlobalGameState::DowntownLoading))
                .run_if(resource_exists::<TileMap<Downtown>>)
                .run_if(common_rscn::tscn_loaded_but_not_spawned::<Downtown>()),
        )
        .add_systems(OnExit(GlobalGameState::DowntownQuitting), despawn);
    }
}

/// Assigned to the root of the apartment scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;

struct DowntownTscnSpawner<'a> {
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    store: &'a GlobalStore,
}

/// The names are stored in the scene file.
/// See the [`Downtown`] implementation of [`SpriteScene`].
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    store: Res<GlobalStore>,

    mut q: Query<&mut TscnTreeHandle<Downtown>>,
) {
    info!("Spawning downtown scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);

    tscn.spawn_into(
        &mut DowntownTscnSpawner {
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
            store: &store,
        },
        &mut cmd,
    );
}

fn despawn(mut cmd: Commands, root: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    let root = root.single();
    cmd.entity(root).despawn_recursive();
}

impl<'a> TscnSpawner for DowntownTscnSpawner<'a> {
    type LocalActionKind = DowntownAction;
    type LocalZoneKind = DowntownTileKind;

    fn on_spawned(
        &mut self,
        cmd: &mut Commands,
        who: Entity,
        NodeName(name): NodeName,
    ) {
        cmd.entity(who)
            .insert(RenderLayers::layer(render_layer::BG));

        match name.as_str() {
            "Downtown" => {
                cmd.entity(who).insert(LayoutEntity);

                let player = crate::actor::spawn_player(
                    cmd,
                    self.asset_server,
                    self.store,
                );
                cmd.entity(who).push_children(&player);
            }
            _ => {}
        }

        trace!("Spawned {name:?} as {who:?} from scene file");
    }

    fn ysort(&mut self, Vec2 { y, .. }: Vec2) -> f32 {
        let (min, max) = Downtown::y_range().into_inner();
        let size = max - min;
        debug_assert!(size > 0.0, "{max} - {min} <= 0.0");

        // we allow for a tiny leeway for positions outside of the bounding box
        ((max - y) / size).clamp(-0.1, 1.1)
    }

    fn add_texture_atlas(
        &mut self,
        layout: TextureAtlasLayout,
    ) -> Handle<TextureAtlasLayout> {
        self.atlases.add(layout)
    }

    fn load_texture(&mut self, path: &str) -> Handle<Image> {
        self.asset_server.load(path.to_owned())
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
