use common_visuals::BeginInterpolationEvent;
use main_game_lib::{
    cutscene::{
        enter_an_elevator::{
            start_with_open_elevator_and_close_it, STEP_TIME_ON_EXIT_ELEVATOR,
        },
        enter_dark_door::EnterDarkDoor,
    },
    top_down::{
        actor::Who, environmental_objects::door::DoorBuilder,
        ActorMovementEvent,
    },
};

use crate::prelude::*;

pub(crate) const THIS_SCENE: WhichTopDownScene =
    WhichTopDownScene::Building1Basement1;

pub(crate) struct Plugin;

/// Important scene struct.
/// We use it as identifiable generic in common logic.
#[derive(TypePath, Default, Debug)]
pub struct Building1Basement1;

impl main_game_lib::rscn::TscnInBevy for Building1Basement1 {
    fn tscn_asset_path() -> String {
        format!("scenes/{}.tscn", THIS_SCENE.snake_case())
    }
}

#[derive(Event, Reflect, Clone, strum::EnumString, Eq, PartialEq)]
pub enum Building1Basement1Action {
    EnterElevator,
    EnterBasement2,
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Building1Basement1Action>();

        app.add_systems(
            OnEnter(THIS_SCENE.loading()),
            rscn::start_loading_tscn::<Building1Basement1>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap>)
                .run_if(
                    rscn::tscn_loaded_but_not_spawned::<Building1Basement1>(),
                ),
        )
        .add_systems(OnExit(THIS_SCENE.leaving()), despawn)
        .add_systems(
            Update,
            enter_the_elevator
                .run_if(on_event_variant(
                    Building1Basement1Action::EnterElevator,
                ))
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(not(in_cutscene())),
        )
        .add_systems(
            Update,
            enter_basement2
                .run_if(on_event_variant(
                    Building1Basement1Action::EnterBasement2,
                ))
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(not(in_cutscene())),
        )
        .add_systems(
            Update,
            watch_entry_to_apartment
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(movement_event_emitted())
                .after(actor::emit_movement_events),
        );
    }
}

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
    zone_to_inspect_label_entity: &'a mut ZoneToInspectLabelEntity,
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

    cmd.remove_resource::<ZoneToInspectLabelEntity>();
}

impl<'a> TscnSpawner for Spawner<'a> {
    type LocalActionKind = Building1Basement1Action;
    type ZoneKind = ZoneTileKind;

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
                let door =
                    DoorBuilder::new(ZoneTileKind::UpperApartmentDoor).build();
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
        zone: Self::ZoneKind,
        entity: Entity,
    ) {
        self.zone_to_inspect_label_entity.insert(zone, entity);
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
        change_global_state_to: THIS_SCENE.leaving(),
        transition: GlobalGameStateTransition::Building1Basement1ToBasement2,
        loading_screen: LoadingScreenSettings { ..default() },
    }
    .spawn(&mut cmd);
}

/// How long does it take for the entity to go transparent
const WALL_FADE_OUT_TRANSITION_DURATION: Duration = from_millis(500);
/// How long does it take for the entity to go to its full color.
const WALL_FADE_IN_TRANSITION_DURATION: Duration = from_millis(1500);

/// Listens to events about entering the
/// [`ZoneTileKind::UpperApartmentWallHidden`].
///
/// When entered, the [`ApartmentWall`] entity is hidden.
fn watch_entry_to_apartment(
    mut movement_events: EventReader<ActorMovementEvent>,
    mut lerp_event: EventWriter<BeginInterpolationEvent>,

    wall: Query<Entity, With<ApartmentWall>>,
) {
    use ZoneTileKind::UpperApartmentWallHidden as TheZone;

    for event in movement_events.read() {
        match event {
            ActorMovementEvent::ZoneEntered {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone: TileKind::Zone(TheZone),
            } => {
                trace!("Hiding apartment wall");
                lerp_event.send(
                    BeginInterpolationEvent::of_color(
                        wall.single(),
                        None,
                        Color::NONE,
                    )
                    .over(WALL_FADE_OUT_TRANSITION_DURATION),
                );
            }
            ActorMovementEvent::ZoneLeft {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone: TileKind::Zone(TheZone),
            } => {
                trace!("Showing apartment wall");
                lerp_event.send(
                    BeginInterpolationEvent::of_color(
                        wall.single(),
                        None,
                        Color::WHITE,
                    )
                    .over(WALL_FADE_IN_TRANSITION_DURATION),
                );
            }

            // we don't care about other events
            _ => {}
        }
    }
}