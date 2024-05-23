use bevy::render::view::RenderLayers;
use bevy_grid_squared::sq;
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_story::Character;
use common_visuals::camera::{render_layer, MainCamera};
use main_game_lib::{
    cutscene::in_cutscene,
    hud::daybar::{DayBar, DayBarDependent, UpdateDayBarEvent},
    top_down::inspect_and_interact::{
        ChangeHighlightedInspectLabelEvent,
        ChangeHighlightedInspectLabelEventConsumer, SpawnLabelBgAndTextParams,
        ZoneToInspectLabelEntity, LIGHT_RED,
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
                enter_building1,
                enter_clinic,
                enter_mall,
                enter_plant_shop,
                enter_sewers,
                enter_twinpeaks_apartment,
            )
                .before(ChangeHighlightedInspectLabelEventConsumer)
                .run_if(on_event::<DowntownAction>())
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

    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut q: Query<&mut TscnTreeHandle<Downtown>>,
) {
    info!("Spawning downtown scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();
    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

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

        #[allow(clippy::single_match)]
        match (name.as_str(), self.transition) {
            ("Downtown", _) => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
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
            | ("ClinicEntrance", ClinicToDowntown) => {
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
            | Self::SewersEntrance
            | Self::MallEntrance
            | Self::ClinicEntrance
            | Self::PlantShopEntrance
            | Self::TwinpeaksApartmentEntrance => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}

fn enter_building1(
    mut cmd: Commands,
    mut action_events: EventReader<DowntownAction>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, DowntownAction::EnterBuilding1));

    if is_triggered {
        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        });

        next_loading_screen_state.set(common_loading_screen::start_state());

        *transition = GlobalGameStateTransition::DowntownToBuilding1PlayerFloor;
        next_state.set(Downtown::quitting());
    }
}

fn enter_mall(
    mut cmd: Commands,
    mut action_events: EventReader<DowntownAction>,
    mut inspect_label_events: EventWriter<ChangeHighlightedInspectLabelEvent>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<DowntownTileKind>,
    >,
    daybar: Res<DayBar>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, DowntownAction::EnterMall));

    if is_triggered {
        if !daybar.is_it_time_for(DayBarDependent::MallOpenHours) {
            if let Some(entity) = zone_to_inspect_label_entity
                .map
                .get(&DowntownTileKind::MallEntrance)
                .copied()
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
                error!("Cannot find mall entrance zone inspect label entity");
            }

            return;
        }

        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        });

        next_loading_screen_state.set(common_loading_screen::start_state());

        *transition = GlobalGameStateTransition::DowntownToMall;
        next_state.set(Downtown::quitting());
    }
}

fn enter_clinic(
    mut cmd: Commands,
    mut action_events: EventReader<DowntownAction>,
    mut inspect_label_events: EventWriter<ChangeHighlightedInspectLabelEvent>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<DowntownTileKind>,
    >,
    daybar: Res<DayBar>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, DowntownAction::EnterClinic));

    if is_triggered {
        if !daybar.is_it_time_for(DayBarDependent::ClinicOpenHours) {
            if let Some(entity) = zone_to_inspect_label_entity
                .map
                .get(&DowntownTileKind::ClinicEntrance)
                .copied()
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
                error!("Cannot find clinic entrance zone inspect label entity");
            }

            return;
        }

        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        });

        next_loading_screen_state.set(common_loading_screen::start_state());

        *transition = GlobalGameStateTransition::DowntownToClinic;
        next_state.set(Downtown::quitting());
    }
}

fn enter_plant_shop(
    mut cmd: Commands,
    mut action_events: EventReader<DowntownAction>,
    mut inspect_label_events: EventWriter<ChangeHighlightedInspectLabelEvent>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<DowntownTileKind>,
    >,
    daybar: Res<DayBar>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, DowntownAction::EnterPlantShop));

    if is_triggered {
        if !daybar.is_it_time_for(DayBarDependent::PlantShopOpenHours) {
            if let Some(entity) = zone_to_inspect_label_entity
                .map
                .get(&DowntownTileKind::PlantShopEntrance)
                .copied()
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
                error!(
                    "Cannot find plant shop entrance zone inspect label entity"
                );
            }

            return;
        }

        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        });

        next_loading_screen_state.set(common_loading_screen::start_state());

        *transition = GlobalGameStateTransition::DowntownToPlantShop;
        next_state.set(Downtown::quitting());
    }
}

fn enter_twinpeaks_apartment(
    mut cmd: Commands,
    mut action_events: EventReader<DowntownAction>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
) {
    let is_triggered = action_events.read().any(|action| {
        matches!(action, DowntownAction::EnterTwinpeaksApartment)
    });

    if is_triggered {
        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        });

        next_loading_screen_state.set(common_loading_screen::start_state());

        *transition = GlobalGameStateTransition::DowntownToTwinpeaksApartment;
        next_state.set(Downtown::quitting());
    }
}

fn enter_sewers(
    mut cmd: Commands,
    mut action_events: EventReader<DowntownAction>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, DowntownAction::EnterSewers));

    if is_triggered {
        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        });

        next_loading_screen_state.set(common_loading_screen::start_state());

        *transition = GlobalGameStateTransition::DowntownToSewers;
        next_state.set(Downtown::quitting());
    }
}
