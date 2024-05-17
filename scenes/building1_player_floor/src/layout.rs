mod watch_entry_to_hallway;

use bevy::render::view::RenderLayers;
use bevy_grid_squared::{sq, GridDirection};
use common_visuals::camera::render_layer;
use main_game_lib::{
    cutscene::enter_an_elevator::{
        start_with_open_elevator_and_close_it, STEP_TIME_ON_EXIT_ELEVATOR,
    },
    hud::daybar::UpdateDayBarEvent,
    top_down::actor::player::TakeAwayPlayerControl,
};
use rscn::{NodeName, TscnSpawner, TscnTree, TscnTreeHandle};
use strum::IntoEnumIterator;
use top_down::{
    actor::{
        self, movement_event_emitted, CharacterBundleBuilder, CharacterExt,
    },
    environmental_objects::{
        self,
        door::{DoorBuilder, DoorOpenCriteria, DoorState},
    },
    inspect_and_interact::ZoneToInspectLabelEntity,
    layout::LAYOUT,
    ActorTarget, TileMap,
};

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Building1PlayerFloor::loading()),
            rscn::start_loading_tscn::<Building1PlayerFloor>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(Building1PlayerFloor::in_loading_state())
                .run_if(resource_exists::<TileMap<Building1PlayerFloor>>)
                .run_if(rscn::tscn_loaded_but_not_spawned::<
                    Building1PlayerFloor,
                >()),
        )
        .add_systems(OnExit(Building1PlayerFloor::quitting()), despawn)
        .add_systems(
            Update,
            (
                watch_entry_to_hallway::system,
                environmental_objects::door::toggle::<Building1PlayerFloor>,
            )
                .run_if(Building1PlayerFloor::in_running_state())
                .run_if(movement_event_emitted::<Building1PlayerFloor>())
                .after(actor::emit_movement_events::<Building1PlayerFloor>),
        );
    }
}

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;
/// Hallway is darkened when the player is in the apartment but once the player
/// approaches the door or is in the hallway, it's lit up.
#[derive(Component)]
pub(crate) struct HallwayEntity;
/// Elevator is a special entity that has a sprite sheet with several frames.
/// It opens when an actor is near it and closes when the actor leaves or
/// enters.
#[derive(Component)]
pub(crate) struct Elevator;
/// Assigned to a sprite that shows Winnie meditating in the chair.
/// This sprite is hidden by default.
#[derive(Component)]
pub(crate) struct MeditatingHint;
/// Same as [`MeditatingHint`] but for the bed.
#[derive(Component)]
pub(crate) struct SleepingHint;

struct Spawner<'a> {
    transition: GlobalGameStateTransition,
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    daybar_event: &'a mut Events<UpdateDayBarEvent>,
    tilemap: &'a mut TileMap<Building1PlayerFloor>,
    zone_to_inspect_label_entity:
        &'a mut ZoneToInspectLabelEntity<Building1PlayerFloorTileKind>,
}

/// The names are stored in the scene file.
/// See the [`Building1PlayerFloor`] implementation of [`SpriteScene`].
fn spawn(
    mut cmd: Commands,
    transition: Res<GlobalGameStateTransition>,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut tilemap: ResMut<TileMap<Building1PlayerFloor>>,
    mut daybar_event: ResMut<Events<UpdateDayBarEvent>>,

    mut q: Query<&mut TscnTreeHandle<Building1PlayerFloor>>,
) {
    info!("Spawning Building1PlayerFloor scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();
    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    tscn.spawn_into(
        &mut Spawner {
            transition: *transition,
            player_entity: player,
            player_builder: &mut player_builder,
            asset_server: &asset_server,
            daybar_event: &mut daybar_event,
            atlases: &mut atlas_layouts,
            tilemap: &mut tilemap,
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
        <Building1PlayerFloor as TopDownScene>::LocalTileKind,
    >>();
}

impl<'a> TscnSpawner for Spawner<'a> {
    type LocalActionKind = Building1PlayerFloorAction;
    type LocalZoneKind = Building1PlayerFloorTileKind;

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

        let came_in_via_elevator = matches!(
            self.transition,
            DowntownToBuilding1PlayerFloor | Building1Basement1ToPlayerFloor
        );

        match name.as_str() {
            "Building1PlayerFloor" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            "Elevator" => {
                cmd.entity(who).insert(Elevator);

                if came_in_via_elevator {
                    let player = self.player_entity;

                    // take away player control for a moment to prevent them
                    // from interacting with the elevator while it's closing
                    cmd.entity(player).insert(TakeAwayPlayerControl);
                    cmd.entity(who).add(move |e: EntityWorldMut| {
                        start_with_open_elevator_and_close_it(player, e)
                    });
                }
            }
            "PlayerApartmentDoor" => {
                let door = DoorBuilder::new(
                    Building1PlayerFloorTileKind::PlayerDoorZone,
                )
                .add_open_criteria(DoorOpenCriteria::Character(
                    common_story::Character::Winnie,
                ))
                .add_open_criteria(DoorOpenCriteria::Character(
                    common_story::Character::Samizdat,
                ))
                .with_initial_state(DoorState::Closed)
                .with_obstacle_when_closed_between(sq(-40, -21), sq(-31, -21))
                .build_and_insert_obstacle(self.tilemap);
                cmd.entity(who).insert(door);
            }
            "WinnieSleeping" => {
                cmd.entity(who).insert(SleepingHint);
            }
            "WinnieMeditating" => {
                cmd.entity(who).insert(MeditatingHint);
            }
            "MeditationSpawn"
                if self.transition == MeditationToBuilding1PlayerFloor =>
            {
                self.player_builder.initial_position(translation.truncate());
                self.player_builder.walking_to(ActorTarget::new(
                    LAYOUT.world_pos_to_square(
                        translation.truncate() + vec2(0.0, -20.0),
                    ),
                ));
                self.player_builder
                    .initial_step_time(STEP_TIME_ONLOAD_FROM_MEDITATION);

                self.daybar_event.send(UpdateDayBarEvent::Meditated);
            }
            "NewGameSpawn"
                if self.transition == NewGameToBuilding1PlayerFloor =>
            {
                self.player_builder.initial_position(translation.truncate());
            }
            "InElevator" if came_in_via_elevator => {
                self.player_builder.initial_position(translation.truncate());
                self.player_builder.walking_to(ActorTarget::new(
                    LAYOUT.world_pos_to_square(translation.truncate())
                        + sq(0, -2),
                ));
                self.player_builder
                    .initial_step_time(STEP_TIME_ON_EXIT_ELEVATOR);
            }
            "AfterSleepSpawn" if self.transition == Sleeping => {
                self.player_builder.initial_position(translation.truncate());
                self.player_builder.initial_direction(GridDirection::Top);
                self.daybar_event.send(UpdateDayBarEvent::NewDay);
            }
            _ => {}
        }
    }

    fn handle_plain_node(
        &mut self,
        cmd: &mut Commands,
        parent: Entity,
        name: String,
        _: rscn::Node,
    ) {
        match name.as_str() {
            "HallwayEntity" => {
                cmd.entity(parent).insert(HallwayEntity);
                cmd.entity(parent).add(|mut w: EntityWorldMut| {
                    w.get_mut::<Sprite>().expect("Sprite").color =
                        PRIMARY_COLOR;
                });
            }
            _ => {
                panic!("Node {name:?} not handled");
            }
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

impl top_down::layout::Tile for Building1PlayerFloorTileKind {
    #[inline]
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    #[inline]
    fn is_zone(&self) -> bool {
        match self {
            Self::BedZone
            | Self::PlayerDoorZone
            | Self::PlayerApartmentZone
            | Self::ElevatorZone
            | Self::HallwayZone
            | Self::MeditationZone
            | Self::TeaZone => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}
