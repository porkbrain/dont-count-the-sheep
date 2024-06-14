use bevy::render::view::RenderLayers;
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_visuals::camera::render_layer;
use main_game_lib::cutscene::in_cutscene;
use rscn::{NodeName, TscnSpawner, TscnTree, TscnTreeHandle};
use strum::IntoEnumIterator;
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlantShop::loading()),
            rscn::start_loading_tscn::<PlantShop>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(PlantShop::in_loading_state())
                .run_if(resource_exists::<TileMap<PlantShop>>)
                .run_if(rscn::tscn_loaded_but_not_spawned::<PlantShop>()),
        )
        .add_systems(OnExit(PlantShop::quitting()), despawn)
        .add_systems(
            Update,
            exit.run_if(on_event::<PlantShopAction>())
                .run_if(PlantShop::in_running_state())
                .run_if(not(in_cutscene())),
        );
    }
}

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;

struct Spawner<'a> {
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    zone_to_inspect_label_entity:
        &'a mut ZoneToInspectLabelEntity<PlantShopTileKind>,

    marie_entity: Entity,
    marie_builder: &'a mut CharacterBundleBuilder,

    bolt_entity: Entity,
    bolt_builder: &'a mut CharacterBundleBuilder,
}

/// The names are stored in the scene file.
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,

    mut q: Query<&mut TscnTreeHandle<PlantShop>>,
) {
    info!("Spawning {PlantShop:?} scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    let marie = cmd.spawn_empty().id();
    let mut marie_builder = common_story::Character::Marie.bundle_builder();

    let bolt = cmd.spawn_empty().id();
    let mut bolt_builder = common_story::Character::Bolt.bundle_builder();

    tscn.spawn_into(
        &mut Spawner {
            player_entity: player,
            player_builder: &mut player_builder,
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
            zone_to_inspect_label_entity: &mut zone_to_inspect_label_entity,

            marie_entity: marie,
            marie_builder: &mut marie_builder,

            bolt_entity: bolt,
            bolt_builder: &mut bolt_builder,
        },
        &mut cmd,
    );

    player_builder.insert_bundle_into(&asset_server, &mut cmd.entity(player));
    marie_builder.insert_bundle_into(&asset_server, &mut cmd.entity(marie));
    bolt_builder.insert_bundle_into(&asset_server, &mut cmd.entity(bolt));

    cmd.insert_resource(zone_to_inspect_label_entity);
}

fn despawn(mut cmd: Commands, root: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    let root = root.single();
    cmd.entity(root).despawn_recursive();

    cmd.remove_resource::<ZoneToInspectLabelEntity<
        <PlantShop as TopDownScene>::LocalTileKind,
    >>();
}

impl<'a> TscnSpawner for Spawner<'a> {
    type LocalActionKind = PlantShopAction;
    type LocalZoneKind = PlantShopTileKind;

    fn on_spawned(
        &mut self,
        cmd: &mut Commands,
        who: Entity,
        NodeName(name): NodeName,
        translation: Vec3,
    ) {
        cmd.entity(who)
            .insert(RenderLayers::layer(render_layer::BG));

        match name.as_str() {
            "PlantShop" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.marie_entity);
                cmd.entity(who).add_child(self.player_entity);
                cmd.entity(who).add_child(self.bolt_entity);
            }
            "Entrance" => {
                self.player_builder.initial_position(translation.truncate());
            }
            "MarieSpawn" => {
                self.marie_builder.initial_position(translation.truncate());
            }
            "BoltSpawn" => {
                self.bolt_builder.initial_position(translation.truncate());
            }
            _ => {}
        }
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

    fn map_zone_to_inspect_label_entity(
        &mut self,
        zone: Self::LocalZoneKind,
        entity: Entity,
    ) {
        self.zone_to_inspect_label_entity.map.insert(zone, entity);
    }
}

impl top_down::layout::Tile for PlantShopTileKind {
    #[inline]
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    #[inline]
    fn is_zone(&self) -> bool {
        match self {
            Self::ExitZone => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}

fn exit(
    mut cmd: Commands,
    mut action_events: EventReader<PlantShopAction>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, PlantShopAction::ExitScene));

    if is_triggered {
        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        });

        next_loading_screen_state.set(common_loading_screen::start_state());

        *transition = GlobalGameStateTransition::PlantShopToDowntown;
        next_state.set(PlantShop::quitting());
    }
}
