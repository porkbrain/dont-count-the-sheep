//! Framework for defining the layout of scenes.
//! Where can the character go? Where are the walls? Where are the immovable
//! objects?

use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_grid_squared::{Square, SquareLayout};
use common_assets::RonLoader;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub fn register<T: IntoMap, S: States>(
    app: &mut App,
    loading: S,
    #[cfg(feature = "dev")] running: S,
) {
    app.init_asset_loader::<RonLoader<Map<T>>>()
        .init_asset::<Map<T>>();

    app.add_systems(OnEnter(loading.clone()), start_loading_map::<T>);
    app.add_systems(
        First,
        try_insert_map_as_resource::<T>.run_if(in_state(loading)),
    );

    #[cfg(feature = "dev")]
    {
        use bevy::input::common_conditions::input_just_pressed;

        app.add_systems(
            OnEnter(running.clone()),
            map_maker::visualize_map::<T>,
        );
        app.add_systems(
            Update,
            map_maker::change_square_kind::<T>
                .run_if(in_state(running.clone())),
        );
        app.add_systems(
            Update,
            map_maker::export_map::<T>
                .run_if(input_just_pressed(KeyCode::Return))
                .run_if(in_state(running)),
        );
    }
}

pub trait IntoMap: 'static + Send + Sync + TypePath {
    fn bounds() -> [i32; 4];

    fn layout() -> &'static SquareLayout;

    fn asset_path() -> &'static str;

    fn cursor_position_to_square(cursor_position: Vec2) -> Square;
}

#[derive(Asset, Resource, Serialize, Deserialize, TypePath)]
pub struct Map<T: IntoMap> {
    squares: HashMap<Square, SquareKind>,
    #[serde(skip)]
    phantom: PhantomData<T>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Default, Eq, PartialEq)]
pub enum SquareKind {
    Wall,
    Object,
    #[default]
    None,
}

impl<T: IntoMap> Map<T> {
    pub fn get(&self, square: &Square) -> Option<SquareKind> {
        self.squares.get(square).copied()
    }
}

/// Tells the game to start loading the map.
/// We need to keep checking for this to be done by calling
/// [`try_insert_map_as_resource`].
fn start_loading_map<T: IntoMap>(
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    let handle: Handle<Map<T>> = assets.load(T::asset_path());
    commands.spawn(handle);
}

/// Run this to wait for the map to be loaded and insert it as a resource.
/// Call it after [`start_loading_map`].
/// Idempotent.
///
/// You should then check for the map as a resource in your systems and continue
/// with your game.
fn try_insert_map_as_resource<T: IntoMap>(
    mut commands: Commands,
    mut map_assets: ResMut<Assets<Map<T>>>,
    map: Query<(Entity, &Handle<Map<T>>)>,
) {
    let Ok((entity, map)) = map.get_single() else {
        // if the map does no longer exist as a component handle, we either did
        // not spawn it or it's already a resource
        // the caller should check for the latter
        return;
    };

    // we cannot call remove straight away because panics - the handle is
    // removed, the map is not loaded yet and asset loader expects it to exist
    if map_assets.get(map).is_some() {
        commands.insert_resource(map_assets.remove(map).unwrap());
        commands.entity(entity).despawn();
    }
}

impl<T: IntoMap> Map<T> {
    #[allow(dead_code)]
    pub fn new(squares: HashMap<Square, SquareKind>) -> Self {
        Self {
            squares,
            phantom: PhantomData,
        }
    }
}

#[cfg(feature = "dev")]
mod map_maker {
    use super::*;
    use bevy::window::PrimaryWindow;

    #[derive(Component)]
    pub(super) struct SquareSprite;

    pub(super) fn visualize_map<T: IntoMap>(
        map: Res<Map<T>>,
        mut commands: Commands,
    ) {
        spawn_grid(&mut commands, &map);
    }

    fn spawn_grid<T: IntoMap>(commands: &mut Commands, map: &Map<T>) {
        for square in bevy_grid_squared::shapes::rectangle(T::bounds()) {
            let world_pos = T::layout().square_to_world_pos(square);

            let kind = map
                .squares
                .get(&square)
                .copied()
                .unwrap_or(SquareKind::None);

            commands.spawn(SquareSprite).insert(SpriteBundle {
                sprite: Sprite {
                    color: kind.color(),
                    // slightly smaller to show borders
                    custom_size: Some(T::layout().square() - 0.25),
                    ..default()
                },
                transform: Transform::from_translation(world_pos.extend(10.0)),
                ..default()
            });
        }
    }

    pub(super) fn change_square_kind<T: IntoMap>(
        windows: Query<&Window, With<PrimaryWindow>>,
        mouse: Res<Input<MouseButton>>,
        squares: Query<Entity, With<SquareSprite>>,
        mut map: ResMut<Map<T>>,
        mut commands: Commands,
    ) {
        let next = mouse.just_pressed(MouseButton::Left);
        let prev = mouse.just_pressed(MouseButton::Right);

        if !next && !prev {
            return;
        }

        let Some(position) = windows.single().cursor_position() else {
            return;
        };

        let needle = T::cursor_position_to_square(position);

        let square_kind = map.squares.entry(needle).or_insert(default());
        *square_kind = if next {
            square_kind.next()
        } else {
            square_kind.prev()
        };

        squares.iter().for_each(|e| commands.entity(e).despawn());

        spawn_grid(&mut commands, &map);
    }

    pub(super) fn export_map<T: IntoMap>(map: Res<Map<T>>) {
        // for internal use only so who cares
        std::fs::write("map.ron", ron::to_string(&*map).unwrap()).unwrap();
    }

    impl SquareKind {
        fn color(self) -> Color {
            match self {
                Self::Wall => Color::rgba(1.0, 0.0, 0.0, 0.5),
                Self::Object => Color::rgba(1.0, 1.0, 1.0, 0.5),
                Self::None => Color::rgba(0.0, 0.0, 0.0, 0.25),
            }
        }

        fn next(self) -> Self {
            match self {
                Self::Wall => Self::Object,
                Self::Object => Self::None,
                Self::None => Self::Wall,
            }
        }

        fn prev(self) -> Self {
            match self {
                Self::Wall => Self::None,
                Self::Object => Self::Wall,
                Self::None => Self::Object,
            }
        }
    }
}
