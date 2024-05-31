use bevy::render::view::RenderLayers;
use common_visuals::camera::render_layer;
use main_game_lib::{
    common_ext::QueryExt,
    cutscene::{enter_dark_door::EnterDarkDoor, in_cutscene, IntoCutscene},
};
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
            OnEnter(Building1Basement2::loading()),
            rscn::start_loading_tscn::<Building1Basement2>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(Building1Basement2::in_loading_state())
                .run_if(resource_exists::<TileMap<Building1Basement2>>)
                .run_if(
                    rscn::tscn_loaded_but_not_spawned::<Building1Basement2>(),
                ),
        )
        .add_systems(OnExit(Building1Basement2::quitting()), despawn)
        .add_systems(
            Update,
            exit.run_if(on_event::<Building1Basement2Action>())
                .run_if(Building1Basement2::in_running_state())
                .run_if(not(in_cutscene())),
        );
    }
}

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;
/// The door sprite that leads to the storage basement.
#[derive(Component)]
pub(crate) struct DoorToStorageBasement;

struct Spawner<'a> {
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    zone_to_inspect_label_entity:
        &'a mut ZoneToInspectLabelEntity<Building1Basement2TileKind>,
}

/// The names are stored in the scene file.
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,

    mut q: Query<&mut TscnTreeHandle<Building1Basement2>>,
) {
    info!("Spawning {Building1Basement2:?} scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    tscn.spawn_into(
        &mut Spawner {
            player_entity: player,
            player_builder: &mut player_builder,
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
            zone_to_inspect_label_entity: &mut zone_to_inspect_label_entity,
        },
        &mut cmd,
    );

    player_builder.insert_bundle_into(&asset_server, &mut cmd.entity(player));

    cmd.insert_resource(zone_to_inspect_label_entity);
}

fn despawn(mut cmd: Commands, root: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    let root = root.single();
    cmd.entity(root).despawn_recursive();

    cmd.remove_resource::<ZoneToInspectLabelEntity<
        <Building1Basement2 as TopDownScene>::LocalTileKind,
    >>();
}

impl<'a> TscnSpawner for Spawner<'a> {
    type LocalActionKind = Building1Basement2Action;
    type LocalZoneKind = Building1Basement2TileKind;

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
            "Building1Basement2" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            "BasementDoor" => {
                cmd.entity(who).insert(DoorToStorageBasement);
            }
            "Exit" => {
                self.player_builder.initial_position(translation.truncate());
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

impl top_down::layout::Tile for Building1Basement2TileKind {
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
    mut action_events: EventReader<Building1Basement2Action>,

    player: Query<Entity, With<Player>>,
    door: Query<Entity, With<DoorToStorageBasement>>,
    points: Query<(&Name, &rscn::Point)>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, Building1Basement2Action::Exit));

    if is_triggered {
        let Some(player) = player.get_single_or_none() else {
            return;
        };

        let door_entrance = points
            .iter()
            .find_map(|(name, rscn::Point(pos))| {
                if name == &Name::new("Exit") {
                    Some(*pos)
                } else {
                    None
                }
            })
            .expect("Missing point for Exit");

        EnterDarkDoor {
            player,
            door: door.single(),
            door_entrance,
            change_global_state_to: Building1Basement2::quitting(),
            transition:
                GlobalGameStateTransition::Building1Basement2ToBasement1,
            loading_screen: default(),
        }
        .spawn(&mut cmd);
    }
}
