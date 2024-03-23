mod watch_entry_to_hallway;

use bevy::render::view::RenderLayers;
use bevy_grid_squared::sq;
use common_rscn::{NodeName, TscnSpawner, TscnTree, TscnTreeHandle};
use common_store::GlobalStore;
use common_top_down::{
    actor::{self, movement_event_emitted},
    environmental_objects::{
        self,
        door::{DoorBuilder, DoorOpenCriteria, DoorState},
    },
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
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

struct ApartmentTscnSpawner<'a> {
    asset_server: &'a AssetServer,
    store: &'a GlobalStore,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    tilemap: &'a mut TileMap<Apartment>,
    zone_to_inspect_label_entity:
        &'a mut ZoneToInspectLabelEntity<ApartmentTileKind>,
}

/// The names are stored in the scene file.
/// See the [`Apartment`] implementation of [`SpriteScene`].
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut tilemap: ResMut<TileMap<Apartment>>,
    store: Res<GlobalStore>,

    mut q: Query<&mut TscnTreeHandle<Apartment>>,
) {
    info!("Spawning apartment scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    tscn.spawn_into(
        &mut ApartmentTscnSpawner {
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
            tilemap: &mut tilemap,
            zone_to_inspect_label_entity: &mut zone_to_inspect_label_entity,
            store: &store,
        },
        &mut cmd,
    );

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
    ) {
        cmd.entity(who)
            .insert(RenderLayers::layer(render_layer::BG));

        match name.as_str() {
            "Apartment" => {
                cmd.entity(who).insert(LayoutEntity);

                // spawn the NPCs and the player to the root node for YSort
                let npcs = crate::actor::spawn_npcs(cmd, self.asset_server);
                cmd.entity(who).push_children(&npcs);

                let player = crate::actor::spawn_player(
                    cmd,
                    self.asset_server,
                    self.store,
                );
                cmd.entity(who).push_children(&player);
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

    fn ysort(&mut self, Vec2 { y, .. }: Vec2) -> f32 {
        let (min, max) = Apartment::y_range().into_inner();
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
