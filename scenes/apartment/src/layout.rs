mod watch_entry_to_hallway;

use bevy::render::view::RenderLayers;
use bevy_grid_squared::sq;
use common_rscn::{NodeName, TscnSpawner, TscnTree, TscnTreeHandle};
use common_top_down::{
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
use common_visuals::camera::render_layer;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{actor::ApartmentAction, prelude::*, Apartment};

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
pub enum ApartmentTileKind {
    /// We want to darken the hallway when the player is in the apartment.
    HallwayZone,
    /// Everything that's in the player's apartment.
    PlayerApartmentZone,
    #[default]
    BedZone,
    ElevatorZone,
    PlayerDoorZone,
    MeditationZone,
    TeaZone,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GlobalGameState::ApartmentLoading),
            common_rscn::start_loading_tscn::<Apartment>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_state(GlobalGameState::ApartmentLoading))
                .run_if(resource_exists::<TileMap<Apartment>>)
                .run_if(common_rscn::tscn_loaded_but_not_spawned::<
                    Apartment,
                >()),
        )
        .add_systems(OnExit(GlobalGameState::ApartmentQuitting), despawn)
        .add_systems(
            Update,
            (
                watch_entry_to_hallway::system,
                environmental_objects::door::toggle::<Apartment>,
            )
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(movement_event_emitted::<Apartment>())
                .after(actor::emit_movement_events::<Apartment>),
        );
    }
}

/// Assigned to the root of the apartment scene.
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

struct ApartmentTscnSpawner<'a> {
    transition: GlobalGameStateTransition,
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    tilemap: &'a mut TileMap<Apartment>,
    zone_to_inspect_label_entity:
        &'a mut ZoneToInspectLabelEntity<ApartmentTileKind>,
}

/// The names are stored in the scene file.
/// See the [`Apartment`] implementation of [`SpriteScene`].
fn spawn(
    mut cmd: Commands,
    transition: Res<GlobalGameStateTransition>,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut tilemap: ResMut<TileMap<Apartment>>,

    mut q: Query<&mut TscnTreeHandle<Apartment>>,
) {
    info!("Spawning apartment scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    let player = cmd.spawn_empty().id();
    let mut player_builder =
        common_story::Character::Winnie.bundle_builder().is_player();

    tscn.spawn_into(
        &mut ApartmentTscnSpawner {
            transition: *transition,
            player_entity: player,
            player_builder: &mut player_builder,
            asset_server: &asset_server,
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
        <Apartment as TopDownScene>::LocalTileKind,
    >>();
}

impl<'a> TscnSpawner for ApartmentTscnSpawner<'a> {
    type LocalActionKind = ApartmentAction;
    type LocalZoneKind = ApartmentTileKind;

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
            "Apartment" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            "Elevator" => {
                cmd.entity(who).insert(Elevator);
            }
            "PlayerApartmentDoor" => {
                let door = DoorBuilder::new(ApartmentTileKind::PlayerDoorZone)
                    .add_open_criteria(DoorOpenCriteria::Character(
                        common_story::Character::Winnie,
                    ))
                    .add_open_criteria(DoorOpenCriteria::Character(
                        common_story::Character::Unnamed,
                    ))
                    .with_initial_state(DoorState::Closed)
                    .with_obstacle_when_closed_between(
                        sq(-40, -21),
                        sq(-31, -21),
                    )
                    .build(self.tilemap);
                cmd.entity(who).insert(door);
            }
            "WinnieSleeping" => {
                cmd.entity(who).insert(SleepingHint);
            }
            "WinnieMeditating" => {
                cmd.entity(who).insert(MeditatingHint);
            }
            "MeditationSpawn" if self.transition == MeditationToApartment => {
                self.player_builder
                    .with_initial_position(translation.truncate());
                self.player_builder.with_walking_to(ActorTarget::new(
                    LAYOUT.world_pos_to_square(
                        translation.truncate() + vec2(0.0, -20.0),
                    ),
                ));
                self.player_builder
                    .with_initial_step_time(STEP_TIME_ONLOAD_FROM_MEDITATION);
            }
            "NewGameSpawn" if self.transition == NewGameToApartment => {
                self.player_builder
                    .with_initial_position(translation.truncate());
            }
            "InElevator" if self.transition == DowntownToApartment => {
                self.player_builder
                    .with_initial_position(translation.truncate());
                self.player_builder.with_walking_to(ActorTarget::new(
                    LAYOUT.world_pos_to_square(translation.truncate())
                        + sq(0, -2),
                ));
                self.player_builder
                    .with_initial_step_time(STEP_TIME_ON_EXIT_ELEVATOR);
            }
            _ => {}
        }

        trace!("Spawned {name:?} as {who:?} from scene file");
    }

    fn handle_plain_node(
        &mut self,
        cmd: &mut Commands,
        parent: Entity,
        name: String,
        _: common_rscn::Node,
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

impl common_top_down::layout::Tile for ApartmentTileKind {
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
