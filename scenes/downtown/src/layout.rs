use bevy::render::view::RenderLayers;
use bevy_grid_squared::{sq, Square};
use common_story::Character;
use common_visuals::camera::{render_layer, MainCamera};
use main_game_lib::{
    cutscene::in_cutscene,
    hud::{
        daybar::{DayBar, DayBarDependent, UpdateDayBarEvent},
        notification::NotificationFifo,
    },
    player_stats::PlayerStats,
    top_down::{
        inspect_and_interact::{
            ChangeHighlightedInspectLabelEvent,
            ChangeHighlightedInspectLabelEventConsumer,
            SpawnLabelBgAndTextParams, ZoneToInspectLabelEntity, LIGHT_RED,
        },
        npc::behaviors::PatrolSequence,
    },
};
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
                .run_if(any_with_component::<MainCamera>)
                .run_if(rscn::tscn_loaded_but_not_spawned::<Downtown>()),
        )
        .add_systems(OnExit(Downtown::quitting()), despawn)
        .add_systems(
            Update,
            (
                enter_building1
                    .run_if(on_event_variant(DowntownAction::EnterBuilding1)),
                enter_clinic
                    .run_if(on_event_variant(DowntownAction::EnterClinic)),
                enter_clinic_ward
                    .run_if(on_event_variant(DowntownAction::EnterClinicWard)),
                enter_mall.run_if(on_event_variant(DowntownAction::EnterMall)),
                enter_plant_shop
                    .run_if(on_event_variant(DowntownAction::EnterPlantShop)),
                enter_sewers
                    .run_if(on_event_variant(DowntownAction::EnterSewers)),
                enter_twinpeaks_apartment.run_if(on_event_variant(
                    DowntownAction::EnterTwinpeaksApartment,
                )),
                enter_compound
                    .run_if(on_event_variant(DowntownAction::EnterCompound)),
            )
                .before(ChangeHighlightedInspectLabelEventConsumer)
                .run_if(Downtown::in_running_state())
                .run_if(not(in_cutscene())),
        );
    }
}

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;

struct Spawner<'a> {
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    camera_translation: &'a mut Vec3,
    player_builder: &'a mut CharacterBundleBuilder,
    player_entity: Entity,
    transition: GlobalGameStateTransition,
    daybar_event: &'a mut Events<UpdateDayBarEvent>,
    zone_to_inspect_label_entity:
        &'a mut ZoneToInspectLabelEntity<DowntownTileKind>,

    samizdat_entity: Entity,
    samizdat_patrol_points: &'a mut Vec<Square>,

    otter_entity: Entity,
    otter_patrol_points: &'a mut Vec<Square>,
}

/// The names are stored in the scene file.
/// See Godot scene file for details.
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    transition: Res<GlobalGameStateTransition>,
    mut daybar_event: ResMut<Events<UpdateDayBarEvent>>,
    mut notifications: ResMut<NotificationFifo>,
    mut player_stats: ResMut<PlayerStats>,

    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut q: Query<&mut TscnTreeHandle<Downtown>>,
) {
    info!("Spawning downtown scene");
    player_stats.visited.downtown(&mut notifications);

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();
    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    let samizdat = cmd.spawn_empty().id();
    let mut samizdat_builder =
        common_story::Character::Samizdat.bundle_builder();
    let mut samizdat_patrol_points = Vec::new();

    let otter = cmd.spawn_empty().id();
    let mut otter_builder = common_story::Character::Otter.bundle_builder();
    let mut otter_patrol_points = Vec::new();

    tscn.spawn_into(
        &mut Spawner {
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
            camera_translation: &mut camera.single_mut().translation,
            daybar_event: &mut daybar_event,
            player_builder: &mut player_builder,
            player_entity: player,
            transition: *transition,
            zone_to_inspect_label_entity: &mut zone_to_inspect_label_entity,

            samizdat_entity: samizdat,
            samizdat_patrol_points: &mut samizdat_patrol_points,

            otter_entity: otter,
            otter_patrol_points: &mut otter_patrol_points,
        },
        &mut cmd,
    );
    cmd.insert_resource(zone_to_inspect_label_entity);

    player_builder.insert_bundle_into(&asset_server, &mut cmd.entity(player));

    assert!(
        !samizdat_patrol_points.is_empty(),
        "No patrol points for samizdat"
    );
    samizdat_builder
        .initial_square(samizdat_patrol_points.first().copied().unwrap())
        .behavior_tree(PatrolSequence {
            wait_at_each: from_millis(7_500),
            points: samizdat_patrol_points,
        });
    samizdat_builder
        .insert_bundle_into(&asset_server, &mut cmd.entity(samizdat));

    assert!(
        !otter_patrol_points.is_empty(),
        "No patrol points for otter"
    );
    otter_builder
        .initial_square(otter_patrol_points.first().copied().unwrap())
        .behavior_tree(PatrolSequence {
            wait_at_each: from_millis(12_000),
            points: otter_patrol_points,
        });
    otter_builder.insert_bundle_into(&asset_server, &mut cmd.entity(otter));
}

fn despawn(mut cmd: Commands, root: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    let root = root.single();
    cmd.entity(root).despawn_recursive();

    cmd.remove_resource::<ZoneToInspectLabelEntity<
        <Downtown as TopDownScene>::LocalTileKind,
    >>();
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

        let position = translation.truncate();

        match (name.as_str(), self.transition) {
            ("Downtown", _) => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
                cmd.entity(who).add_child(self.samizdat_entity);
                cmd.entity(who).add_child(self.otter_entity);
            }

            // transitions
            (
                "PlayerApartmentBuildingEntrance",
                Building1Basement1ToDowntown | Building1PlayerFloorToDowntown,
            )
            | ("MallEntrance", MallToDowntown)
            | ("TwinpeaksApartmentEntrance", TwinpeaksApartmentToDowntown)
            | ("PlantShopEntrance", PlantShopToDowntown)
            | ("SewersEntrance", SewersToDowntown)
            | ("CompoundEntrance", CompoundToDowntown)
            | ("ClinicExit", ClinicToDowntown | ClinicWardToDowntown) => {
                self.camera_translation.x = position.x;
                self.camera_translation.y = position.y;
                // we multiply by 4 because winnie is walking across 2 tiles and
                // we want her to be extra extra slow because it looks better
                self.player_builder
                    .initial_step_time(Character::Winnie.slow_step_time() * 4);
                self.player_builder.initial_position(position);
                self.player_builder.walking_to(top_down::ActorTarget::new(
                    LAYOUT.world_pos_to_square(position) + sq(0, -2),
                ));

                self.daybar_event.send(UpdateDayBarEvent::ChangedScene);
            }

            (s, _) if s.starts_with("SamizdatPatrolPoint") => {
                self.samizdat_patrol_points
                    .push(LAYOUT.world_pos_to_square(translation.truncate()));
            }
            (s, _) if s.starts_with("OtterPatrolPoint") => {
                self.otter_patrol_points
                    .push(LAYOUT.world_pos_to_square(translation.truncate()));
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

impl top_down::layout::Tile for DowntownTileKind {
    #[inline]
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    #[inline]
    fn is_zone(&self) -> bool {
        match self {
            Self::Building1Entrance
            | Self::CompoundEntrance
            | Self::SewersEntrance
            | Self::MallEntrance
            | Self::ClinicEntrance
            | Self::ClinicWardEntrance
            | Self::PlantShopEntrance
            | Self::TwinpeaksApartmentEntrance => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}

fn enter_building1(mut transition_params: TransitionParams) {
    transition_params
        .begin(GlobalGameStateTransition::DowntownToBuilding1PlayerFloor);
}

fn enter_mall(
    mut transition_params: TransitionParams,
    mut inspect_label_events: EventWriter<ChangeHighlightedInspectLabelEvent>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<DowntownTileKind>,
    >,
    daybar: Res<DayBar>,
) {
    if !daybar.is_it_time_for(DayBarDependent::MallOpenHours) {
        show_label_closed(
            &zone_to_inspect_label_entity,
            &mut inspect_label_events,
            &DowntownTileKind::MallEntrance,
        );

        return;
    }

    transition_params.begin(GlobalGameStateTransition::DowntownToMall);
}

fn enter_clinic(
    mut transition_params: TransitionParams,
    mut inspect_label_events: EventWriter<ChangeHighlightedInspectLabelEvent>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<DowntownTileKind>,
    >,
    daybar: Res<DayBar>,
) {
    if !daybar.is_it_time_for(DayBarDependent::ClinicOpenHours) {
        show_label_closed(
            &zone_to_inspect_label_entity,
            &mut inspect_label_events,
            &DowntownTileKind::ClinicEntrance,
        );

        return;
    }

    transition_params.begin(GlobalGameStateTransition::DowntownToClinic);
}

fn enter_clinic_ward(
    mut transition_params: TransitionParams,
    mut inspect_label_events: EventWriter<ChangeHighlightedInspectLabelEvent>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<DowntownTileKind>,
    >,
    daybar: Res<DayBar>,
) {
    if !daybar.is_it_time_for(DayBarDependent::ClinicOpenHours) {
        show_label_closed(
            &zone_to_inspect_label_entity,
            &mut inspect_label_events,
            &DowntownTileKind::ClinicWardEntrance,
        );

        return;
    }

    transition_params.begin(GlobalGameStateTransition::DowntownToClinicWard);
}

fn enter_plant_shop(
    mut transition_params: TransitionParams,
    mut inspect_label_events: EventWriter<ChangeHighlightedInspectLabelEvent>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<DowntownTileKind>,
    >,
    daybar: Res<DayBar>,
) {
    if !daybar.is_it_time_for(DayBarDependent::PlantShopOpenHours) {
        show_label_closed(
            &zone_to_inspect_label_entity,
            &mut inspect_label_events,
            &DowntownTileKind::PlantShopEntrance,
        );

        return;
    }

    transition_params.begin(GlobalGameStateTransition::DowntownToPlantShop);
}

fn enter_twinpeaks_apartment(mut transition_params: TransitionParams) {
    transition_params
        .begin(GlobalGameStateTransition::DowntownToTwinpeaksApartment);
}

fn enter_sewers(mut transition_params: TransitionParams) {
    transition_params.begin(GlobalGameStateTransition::DowntownToSewers);
}

fn enter_compound(mut transition_params: TransitionParams) {
    transition_params.begin(GlobalGameStateTransition::DowntownToCompound);
}

fn show_label_closed(
    zone_to_inspect_label_entity: &ZoneToInspectLabelEntity<DowntownTileKind>,
    inspect_label_events: &mut EventWriter<ChangeHighlightedInspectLabelEvent>,
    zone_kind: &DowntownTileKind,
) {
    if let Some(entity) =
        zone_to_inspect_label_entity.map.get(zone_kind).copied()
    {
        inspect_label_events.send(ChangeHighlightedInspectLabelEvent {
            entity,
            spawn_params: SpawnLabelBgAndTextParams {
                highlighted: true,
                overwrite_font_color: Some(LIGHT_RED),
                // LOCALIZATION
                overwrite_display_text: Some("(closed)".to_string()),
            },
        });
    } else {
        error!("Cannot find clinic entrance zone for {zone_kind:?}");
    }
}
