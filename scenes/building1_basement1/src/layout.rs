mod watch_entry_to_apartment;

use bevy::render::view::RenderLayers;
use bevy_grid_squared::sq;
use common_loading_screen::LoadingScreenSettings;
use common_visuals::camera::{render_layer, MainCamera};
use main_game_lib::{
    common_ext::QueryExt,
    cutscene::{
        self,
        enter_an_elevator::{
            start_with_open_elevator_and_close_it, STEP_TIME_ON_EXIT_ELEVATOR,
        },
        enter_dark_door::EnterDarkDoor,
        in_cutscene, IntoCutscene,
    },
    dialog::DialogGraph,
    top_down::{
        actor::{self, movement_event_emitted, player::TakeAwayPlayerControl},
        environmental_objects::{self, door::DoorBuilder},
    },
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
            OnEnter(Building1Basement1::loading()),
            rscn::start_loading_tscn::<Building1Basement1>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(Building1Basement1::in_loading_state())
                .run_if(resource_exists::<TileMap<Building1Basement1>>)
                .run_if(
                    rscn::tscn_loaded_but_not_spawned::<Building1Basement1>(),
                ),
        )
        .add_systems(OnExit(Building1Basement1::quitting()), despawn)
        .add_systems(
            Update,
            environmental_objects::door::toggle::<Building1Basement1>
                .run_if(Building1Basement1::in_running_state())
                .run_if(movement_event_emitted::<Building1Basement1>())
                .after(actor::emit_movement_events::<Building1Basement1>),
        )
        .add_systems(
            Update,
            enter_the_elevator
                .run_if(on_event_variant(
                    Building1Basement1Action::EnterElevator,
                ))
                .run_if(Building1Basement1::in_running_state())
                .run_if(not(in_cutscene())),
        )
        .add_systems(
            Update,
            enter_basement2
                .run_if(on_event_variant(
                    Building1Basement1Action::EnterBasement2,
                ))
                .run_if(Building1Basement1::in_running_state())
                .run_if(not(in_cutscene())),
        )
        .add_systems(
            Update,
            watch_entry_to_apartment::system
                .run_if(Building1Basement1::in_running_state())
                .run_if(movement_event_emitted::<Building1Basement1>())
                .after(actor::emit_movement_events::<Building1Basement1>),
        );
    }
}

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;
/// Elevator is a special entity that has a sprite sheet with several frames.
/// It opens when an actor is near it and closes when the actor leaves or
/// enters.
#[derive(Component)]
pub(crate) struct Elevator;
/// The door sprite that leads to the storage basement.
#[derive(Component)]
pub(crate) struct DoorToStorageBasement;
/// There's a wall that separates an apartment from the hallway.
/// This door gets hidden when the player is near or in the apartment.
#[derive(Component)]
pub(crate) struct ApartmentWall;

struct Spawner<'a> {
    transition: GlobalGameStateTransition,
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    zone_to_inspect_label_entity:
        &'a mut ZoneToInspectLabelEntity<Building1Basement1TileKind>,
}

/// The names are stored in the scene file.
fn spawn(
    mut cmd: Commands,
    transition: Res<GlobalGameStateTransition>,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,

    mut q: Query<&mut TscnTreeHandle<Building1Basement1>>,
) {
    info!("Spawning {Building1Basement1:?} scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();
    player_builder.initial_step_time(STEP_TIME_ON_EXIT_ELEVATOR);

    tscn.spawn_into(
        &mut Spawner {
            transition: *transition,
            player_entity: player,
            player_builder: &mut player_builder,
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
            zone_to_inspect_label_entity: &mut zone_to_inspect_label_entity,
        },
        &mut cmd,
    );

    player_builder.walking_to_from_initial_position(sq(0, -2));
    player_builder.insert_bundle_into(&asset_server, &mut cmd.entity(player));

    cmd.insert_resource(zone_to_inspect_label_entity);
}

fn despawn(mut cmd: Commands, root: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    let root = root.single();
    cmd.entity(root).despawn_recursive();

    cmd.remove_resource::<ZoneToInspectLabelEntity<
        <Building1Basement1 as TopDownScene>::LocalTileKind,
    >>();
}

impl<'a> TscnSpawner for Spawner<'a> {
    type LocalActionKind = Building1Basement1Action;
    type LocalZoneKind = Building1Basement1TileKind;

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

        match name.as_str() {
            "Building1Basement1" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            "Elevator" => {
                cmd.entity(who).insert(Elevator);

                if self.transition == Building1PlayerFloorToBuilding1Basement1 {
                    let player = self.player_entity;

                    // take away player control for a moment to prevent them
                    // from interacting with the elevator while it's closing
                    cmd.entity(player).insert(TakeAwayPlayerControl);
                    cmd.entity(who).add(move |e: EntityWorldMut| {
                        start_with_open_elevator_and_close_it(player, e)
                    });
                }
            }
            "InElevator"
                if self.transition
                    == Building1PlayerFloorToBuilding1Basement1 =>
            {
                self.player_builder.initial_position(translation.truncate());
            }
            "BasementExit"
                if self.transition == Building1Basement2ToBasement1 =>
            {
                self.player_builder.initial_position(translation.truncate());
            }
            "DoorToBasement2" => {
                cmd.entity(who).insert(DoorToStorageBasement);
            }
            "ApartmentWall" => {
                cmd.entity(who).insert(ApartmentWall);
            }
            "DoorToTheUpperApartment" => {
                let door = DoorBuilder::new(
                    Building1Basement1TileKind::UpperApartmentDoorZone,
                )
                .build::<Building1Basement1>();
                cmd.entity(who).insert(door);
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

impl top_down::layout::Tile for Building1Basement1TileKind {
    #[inline]
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    #[inline]
    fn is_zone(&self) -> bool {
        match self {
            Self::UpperApartmentWallHiddenZone
            | Self::UpperApartmentDoorZone
            | Self::BasementDoorZone
            | Self::ElevatorZone => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}

/// By entering the elevator, the player can this scene.
fn enter_the_elevator(
    mut cmd: Commands,
    mut assets: ResMut<Assets<DialogGraph>>,

    player: Query<Entity, With<Player>>,
    elevator: Query<Entity, With<Elevator>>,
    camera: Query<Entity, With<MainCamera>>,
    points: Query<(&Name, &rscn::Point)>,
) {
    let Some(player) = player.get_single_or_none() else {
        error!("Cannot enter the elevator because the Player is missing");
        return;
    };

    let point_in_elevator = {
        let (_, rscn::Point(pos)) = points
            .iter()
            .find(|(name, _)| **name == Name::new("InElevator"))
            .expect("InElevator point not found");

        *pos
    };

    cutscene::enter_an_elevator::spawn(
        &mut cmd,
        &mut assets,
        player,
        elevator.single(),
        camera.single(),
        point_in_elevator,
        &[
            (
                GlobalGameStateTransition::Building1Basement1ToPlayerFloor,
                "go to first floor",
            ),
            (
                GlobalGameStateTransition::Building1Basement1ToDowntown,
                "go to downtown",
            ),
        ],
    );
}

/// Goes to the next basement level.
fn enter_basement2(
    mut cmd: Commands,

    player: Query<Entity, With<Player>>,
    door: Query<Entity, With<DoorToStorageBasement>>,
    points: Query<(&Name, &rscn::Point)>,
) {
    let Some(player) = player.get_single_or_none() else {
        return;
    };

    let door_entrance = points
        .iter()
        .find_map(|(name, rscn::Point(pos))| {
            if name == &Name::new("BasementExit") {
                Some(*pos)
            } else {
                None
            }
        })
        .expect("Missing point for BasementExit");

    EnterDarkDoor {
        player,
        door: door.single(),
        door_entrance,
        change_global_state_to: Building1Basement1::quitting(),
        transition: GlobalGameStateTransition::Building1Basement1ToBasement2,
        loading_screen: LoadingScreenSettings { ..default() },
    }
    .spawn(&mut cmd);
}
