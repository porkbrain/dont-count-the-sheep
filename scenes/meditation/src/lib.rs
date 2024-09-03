#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod cameras;
mod consts;
mod hoshi;
mod prelude;
mod ui;
mod zindex;

use bevy::{render::view::RenderLayers, utils::Instant};
use common_assets::{store::AssetList, AssetStore};
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_visuals::camera::render_layer;
use prelude::*;
use rscn::{NodeName, TscnSpawner, TscnTree, TscnTreeHandle};

/// Important scene struct.
/// Identifies anything that's related to meditation.
#[derive(TypePath, Debug, Default)]
struct Meditation;

pub fn add(app: &mut App) {
    info!("Adding meditation to app");

    debug!("Adding plugins");

    app.add_plugins((ui::Plugin, hoshi::Plugin, cameras::Plugin));

    debug!("Adding assets");

    app.add_systems(
        OnEnter(GlobalGameState::LoadingMeditation),
        common_assets::store::insert_as_resource::<Meditation>,
    );
    app.add_systems(
        OnExit(GlobalGameState::QuittingMeditation),
        common_assets::store::remove_as_resource::<Meditation>,
    );

    debug!("Adding visuals");

    app.add_systems(
        Update,
        common_visuals::systems::flicker
            .run_if(in_state(GlobalGameState::MeditationInMenu)),
    );

    debug!("Adding physics");

    app.add_systems(
        FixedUpdate,
        common_physics::systems::apply_velocity
            .run_if(in_state(GlobalGameState::InGameMeditation)),
    );

    debug!("Adding game loop");

    // 1. start the spawning process (the loading screen is already started)
    app.add_systems(
        OnEnter(GlobalGameState::LoadingMeditation),
        rscn::start_loading_tscn::<LevelOne>,
    );
    // 2. when everything is loaded, finish the loading process by transitioning
    //    to the next loading state (this will also spawn the camera)
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(in_state(GlobalGameState::LoadingMeditation))
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // 3. ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_game.run_if(in_state(GlobalGameState::LoadingMeditation)),
    );

    app.add_systems(OnExit(GlobalGameState::QuittingMeditation), despawn);
    app.add_systems(
        Last,
        all_cleaned_up.run_if(in_state(GlobalGameState::QuittingMeditation)),
    );

    info!("Added meditation to app");
}

struct LevelOne;

impl main_game_lib::rscn::TscnInBevy for LevelOne {
    fn tscn_asset_path() -> String {
        format!("scenes/meditation_lvl1.tscn")
    }
}

struct Spawner<'a> {
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
}

impl<'a> TscnSpawner for Spawner<'a> {
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
            "HoshiSpawn" => {
                info!("Hoshi spawn point");
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
}

fn despawn(mut cmd: Commands) {
    debug!("Despawning resources");
}

fn finish_when_everything_loaded(
    mut cmd: Commands,
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    asset_store: Res<AssetStore<Meditation>>,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,

    mut q: Query<&mut TscnTreeHandle<LevelOne>>,
) {
    if !asset_store.are_all_loaded(&asset_server) {
        return;
    }

    debug!("All assets loaded");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    tscn.spawn_into_world(
        &mut Spawner {
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
        },
        &mut cmd,
    );

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_game(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering meditation game");
    next_state.set(GlobalGameState::InGameMeditation);
}

fn all_cleaned_up(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
    settings: Res<LoadingScreenSettings>,

    mut since: Local<Option<Instant>>,
) {
    // this is reset to None when we're done with the exit animation
    let elapsed = since.get_or_insert_with(Instant::now).elapsed();
    if elapsed < settings.fade_loading_screen_in {
        return;
    }

    info!("Leaving meditation game");

    // reset local state for next time
    *since = None;

    // be a good guy and don't invade other game loops with "Enter"
    controls.consume(&GlobalAction::Interact);

    use GlobalGameStateTransition::*;
    match *transition {
        RestartMeditation => {
            next_state.set(GlobalGameState::LoadingMeditation);
        }
        MeditationToBuilding1PlayerFloor => {
            next_state.set(WhichTopDownScene::Building1PlayerFloor.loading());
        }
        _ => {
            unreachable!("Invalid meditation transition {transition:?}");
        }
    }
}

impl AssetList for Meditation {
    fn folders() -> &'static [&'static str] {
        &[common_assets::meditation::FOLDER]
    }
}
