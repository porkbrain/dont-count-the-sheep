#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod assets;
mod bedroom;
mod cameras;
mod characters;
mod consts;
mod layout;
mod prelude;
mod zindex;

use bevy::input::common_conditions::input_just_pressed;
use bevy_grid_squared::{Square, SquareLayout};
use common_assets::RonLoader;
use layout::IntoMap;
use lazy_static::lazy_static;
use main_game_lib::{
    GlobalGameStateTransition, GlobalGameStateTransitionStack, PIXEL_ZOOM,
    VISIBLE_HEIGHT, VISIBLE_WIDTH,
};
use prelude::*;

#[derive(Component, TypePath)]
struct Apartment;

lazy_static! {
    static ref LAYOUT: SquareLayout = SquareLayout {
        square_size: 4.0,
        origin: vec2(356.0, 175.0).from_top_left_into_centered(),
    };
}

trait Vec2Ext {
    fn from_top_left_into_centered(self) -> Self;
}

impl Vec2Ext for Vec2 {
    fn from_top_left_into_centered(self) -> Self {
        Self::new(self.x - VISIBLE_WIDTH / 2.0, -self.y + VISIBLE_HEIGHT / 2.0)
    }
}

impl IntoMap for Apartment {
    fn bounds() -> [i32; 4] {
        [-40, 40, -20, 20]
    }

    fn asset_path() -> &'static str {
        assets::APARTMENT_MAP
    }

    fn layout() -> &'static SquareLayout {
        &LAYOUT
    }

    fn cursor_position_to_square(p: Vec2) -> Square {
        Self::layout()
            .world_pos_to_square((p / PIXEL_ZOOM).from_top_left_into_centered())
    }
}

pub fn add(app: &mut App) {
    info!("Adding apartment to app");

    app.init_asset_loader::<RonLoader<layout::Map<Apartment>>>()
        .init_asset::<layout::Map<Apartment>>();

    debug!("Adding plugins");

    app.add_plugins((cameras::Plugin, bedroom::Plugin, characters::Plugin));

    debug!("Adding game loop");

    app.add_systems(
        OnEnter(GlobalGameState::ApartmentLoading),
        (spawn, layout::start_loading_map::<Apartment>),
    );
    app.add_systems(
        First,
        layout::try_insert_map_as_resource::<Apartment>
            .run_if(in_state(GlobalGameState::ApartmentLoading)),
    );
    app.add_systems(
        Last,
        all_loaded.run_if(in_state(GlobalGameState::ApartmentLoading)),
    );
    app.add_systems(
        OnExit(GlobalGameState::ApartmentLoading),
        layout::visualize_map::<Apartment>,
    );

    app.add_systems(
        Update,
        (close_game, open_meditation)
            .run_if(in_state(GlobalGameState::InApartment)),
    );
    // TODO: dev only
    app.add_systems(
        Update,
        layout::change_square_kind::<Apartment>
            .run_if(in_state(GlobalGameState::InApartment)),
    );
    app.add_systems(
        Update,
        layout::export_map::<Apartment>
            .run_if(input_just_pressed(KeyCode::Return))
            .run_if(in_state(GlobalGameState::InApartment)),
    );

    app.add_systems(OnEnter(GlobalGameState::ApartmentQuitting), despawn);
    app.add_systems(
        Last,
        all_cleaned_up.run_if(in_state(GlobalGameState::ApartmentQuitting)),
    );

    info!("Added apartment to app");
}

/// Temp. solution: press ESC to quit.
fn close_game(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    keyboard: ResMut<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        stack.push(GlobalGameStateTransition::ApartmentQuittingToExit);
        next_state.set(GlobalGameState::ApartmentQuitting);
    }
}

/// Temp. solution: press M to open meditation.
fn open_meditation(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    keyboard: ResMut<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::M) {
        stack.push(
            GlobalGameStateTransition::ApartmentQuittingToMeditationLoading,
        );
        next_state.set(GlobalGameState::ApartmentQuitting);
    }
}

fn spawn(mut commands: Commands) {
    debug!("Spawning resources ClearColor");

    commands.insert_resource(ClearColor(PRIMARY_COLOR));
}

fn despawn(mut commands: Commands) {
    debug!("Despawning resources ClearColor");

    commands.remove_resource::<ClearColor>();
}

fn all_loaded(
    map: Option<Res<layout::Map<Apartment>>>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
    if map.is_none() {
        return;
    }

    info!("Entering apartment");

    next_state.set(GlobalGameState::InApartment);
}

fn all_cleaned_up(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
    info!("Leaving apartment");

    match stack.pop_next_for(GlobalGameState::ApartmentQuitting) {
        // possible restart or change of game loop
        Some(next) => next_state.set(next),
        None => {
            unreachable!(
                "There's nowhere to transition from ApartmentQuitting"
            );
        }
    }
}
