use bevy::render::view::RenderLayers;
use bevy_grid_squared::sq;
use common_visuals::camera::render_layer;
use rscn::{NodeName, TscnSpawner, TscnTree, TscnTreeHandle};
use strum::IntoEnumIterator;
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    layout::LAYOUT,
    TileMap,
};

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Downtown::loading()),
            rscn::start_loading_tscn::<Downtown>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(Downtown::in_loading_state())
                .run_if(resource_exists::<TileMap<Downtown>>)
                .run_if(rscn::tscn_loaded_but_not_spawned::<Downtown>()),
        )
        .add_systems(OnExit(Downtown::quitting()), despawn);
    }
}

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;

struct Spawner<'a> {
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    player_builder: &'a mut CharacterBundleBuilder,
    player_entity: Entity,
    transition: GlobalGameStateTransition,
}

/// The names are stored in the scene file.
/// See the [`Downtown`] implementation of [`SpriteScene`].
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    transition: Res<GlobalGameStateTransition>,

    mut q: Query<&mut TscnTreeHandle<Downtown>>,
) {
    info!("Spawning downtown scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);

    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    tscn.spawn_into(
        &mut Spawner {
            asset_server: &asset_server,
            transition: *transition,
            atlases: &mut atlas_layouts,
            player_entity: player,
            player_builder: &mut player_builder,
        },
        &mut cmd,
    );

    player_builder.insert_bundle_into(&asset_server, &mut cmd.entity(player));
}

fn despawn(mut cmd: Commands, root: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    let root = root.single();
    cmd.entity(root).despawn_recursive();
}

impl<'a> TscnSpawner for Spawner<'a> {
    type LocalActionKind = DowntownAction;
    type LocalZoneKind = DowntownTileKind;

    fn on_spawned(
        &mut self,
        cmd: &mut Commands,
        who: Entity,
        NodeName(name): NodeName,
        translation: Vec3,
    ) {
        use GlobalGameStateTransition::*;
        cmd.entity(who)
            .insert(RenderLayers::layer(render_layer::BG));

        #[allow(clippy::single_match)]
        match name.as_str() {
            "Downtown" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            "PlayerApartmentBuildingEntrance"
                if matches!(
                    self.transition,
                    Building1Basement1ToDowntown
                        | Building1PlayerFloorToDowntown
                ) =>
            {
                self.player_builder.initial_position(translation.truncate());
                self.player_builder.walking_to(top_down::ActorTarget::new(
                    LAYOUT.world_pos_to_square(translation.truncate())
                        + sq(0, -2),
                ));
            }
            _ => {}
        }

        trace!("Spawned {name:?} as {who:?} from scene file");
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

impl top_down::layout::Tile for DowntownTileKind {
    #[inline]
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    #[inline]
    fn is_zone(&self) -> bool {
        match self {
            Self::MallEntrance | Self::PlayerHouseEntrance => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}
