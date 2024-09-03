use bevy::render::view::RenderLayers;
use common_visuals::camera::render_layer;
use main_game_lib::{
    cutscene::in_cutscene, hud::notification::NotificationFifo,
    player_stats::PlayerStats,
};
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

const THIS_SCENE: WhichTopDownScene = WhichTopDownScene::PlantShop;

#[derive(TypePath, Default, Debug)]
struct PlantShop;

impl main_game_lib::rscn::TscnInBevy for PlantShop {
    fn tscn_asset_path() -> String {
        format!("scenes/{}.tscn", THIS_SCENE.snake_case())
    }
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(THIS_SCENE.loading()),
            rscn::start_loading_tscn::<PlantShop>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap>)
                .run_if(rscn::tscn_loaded_but_not_spawned::<PlantShop>()),
        )
        .add_systems(OnExit(THIS_SCENE.leaving()), despawn)
        .add_systems(
            Update,
            exit.run_if(on_event_variant(TopDownAction::Exit))
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(not(in_cutscene())),
        );
    }
}

struct Spawner<'a> {
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,

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
    mut notifications: ResMut<NotificationFifo>,
    mut player_stats: ResMut<PlayerStats>,

    mut q: Query<&mut TscnTreeHandle<PlantShop>>,
) {
    info!("Spawning {PlantShop:?} scene");
    player_stats.visited.plant_shop(&mut notifications);

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    let marie = cmd.spawn_empty().id();
    let mut marie_builder = common_story::Character::Marie.bundle_builder();

    let bolt = cmd.spawn_empty().id();
    let mut bolt_builder = common_story::Character::Bolt.bundle_builder();

    tscn.spawn_into(
        &mut cmd,
        &mut atlas_layouts,
        &asset_server,
        &mut TopDownTsncSpawner::new(
            &mut zone_to_inspect_label_entity,
            &mut Spawner {
                player_entity: player,
                player_builder: &mut player_builder,

                marie_entity: marie,
                marie_builder: &mut marie_builder,

                bolt_entity: bolt,
                bolt_builder: &mut bolt_builder,
            },
        ),
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

    cmd.remove_resource::<ZoneToInspectLabelEntity>();
}

impl<'a> TscnSpawnHooks for Spawner<'a> {
    fn handle_2d_node(
        &mut self,
        cmd: &mut Commands,
        descriptions: &mut EntityDescriptionMap,
        _parent: Option<(Entity, NodeName)>,
        (who, NodeName(name)): (Entity, NodeName),
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
                let translation = descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.player_builder.initial_position(translation);
            }
            "MarieSpawn" => {
                let translation = descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.marie_builder.initial_position(translation);
            }
            "BoltSpawn" => {
                let translation = descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.bolt_builder.initial_position(translation);
            }
            _ => {}
        }
    }
}

fn exit(mut transition_params: TransitionParams) {
    transition_params.begin(GlobalGameStateTransition::PlantShopToDowntown);
}
