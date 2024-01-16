//! Framework for defining the layout of scenes.
//! Where can the character go? Where are the walls? Where are the immovable
//! objects?

#![deny(missing_docs)]

use std::marker::PhantomData;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_grid_squared::{Square, SquareLayout};
use common_assets::RonLoader;
use serde::{Deserialize, Serialize};

/// Registers layout map for `T` where `T` is a type implementing [`IntoMap`].
/// This would be your level layout.
///
/// If the `dev` feature is enabled, you can press `Enter` to export the map
/// to `map.ron` in the current directory.
/// We draw an overlay with tiles that you can edit with left and right mouse
/// buttons.
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

/// Some map.
pub trait IntoMap: 'static + Send + Sync + TypePath {
    /// Size in number of tiles.
    /// `[left, right, top, bottom]`
    fn bounds() -> [i32; 4];

    /// How large is a tile and how do we translate between world coordinates
    /// and tile coordinates?
    fn layout() -> &'static SquareLayout;

    /// Path to the map .ron asset.
    fn asset_path() -> &'static str;

    /// Convert a cursor position to a tile.
    /// This cannot be done with the layout because cursor is relative to the
    /// window size and starts at top left corner.
    fn cursor_position_to_square(cursor_position: Vec2) -> Square;

    /// Given a position on the map, add a z coordinate.
    /// Custom implementations can be used to add z index based on y coordinate.
    fn extend_z(v: Vec2) -> Vec3 {
        v.extend(0.0)
    }

    /// Whether the given square is inside the map.
    fn contains(square: Square) -> bool {
        let [min_x, max_x, min_y, max_y] = Self::bounds();

        square.x >= min_x
            && square.x <= max_x
            && square.y >= min_y
            && square.y <= max_y
    }
}

/// Holds the tiles in a hash map.
#[derive(Asset, Resource, Serialize, Deserialize, TypePath)]
pub struct Map<T: IntoMap> {
    squares: HashMap<Square, SquareKind>,
    #[serde(skip)]
    phantom: PhantomData<T>,
}

/// What kind of tiles do we support?
#[derive(Clone, Copy, Serialize, Deserialize, Default, Eq, PartialEq)]
pub enum SquareKind {
    /// No tile.
    /// Preferably, don't put these into the hash map.
    #[default]
    None,
    /// A wall that cannot be passed.
    Wall,
    /// An object that blocks.
    Object,
    /// A space that can be depended on by the game logic.
    /// You can match the zone number to a check whether the character is in
    /// a tile of that zone.
    Zone(u8),
}

impl<T: IntoMap> Map<T> {
    /// Get the kind of a tile.
    pub fn get(&self, square: &Square) -> Option<SquareKind> {
        self.squares.get(square).copied()
    }
}

/// Tells the game to start loading the map.
/// We need to keep checking for this to be done by calling
/// [`try_insert_map_as_resource`].
fn start_loading_map<T: IntoMap>(mut cmd: Commands, assets: Res<AssetServer>) {
    let handle: Handle<Map<T>> = assets.load(T::asset_path());
    cmd.spawn(handle);
}

/// Run this to wait for the map to be loaded and insert it as a resource.
/// Call it after [`start_loading_map`].
/// Idempotent.
///
/// You should then check for the map as a resource in your systems and continue
/// with your game.
fn try_insert_map_as_resource<T: IntoMap>(
    mut cmd: Commands,
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
        cmd.insert_resource(map_assets.remove(map).unwrap());
        cmd.entity(entity).despawn();
    }
}

impl<T: IntoMap> Map<T> {
    /// Create a new map with the given squares.
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
    use bevy::window::PrimaryWindow;

    use super::*;

    #[derive(Component)]
    pub(super) struct SquareSprite;

    pub(super) fn visualize_map<T: IntoMap>(
        mut cmd: Commands,
        map: Res<Map<T>>,
    ) {
        spawn_grid(&mut cmd, &map);
    }

    fn spawn_grid<T: IntoMap>(cmd: &mut Commands, map: &Map<T>) {
        for square in bevy_grid_squared::shapes::rectangle(T::bounds()) {
            let world_pos = T::layout().square_to_world_pos(square);

            let kind = map
                .squares
                .get(&square)
                .copied()
                .unwrap_or(SquareKind::None);

            cmd.spawn((SquareSprite, Name::new("Debug square"))).insert(
                SpriteBundle {
                    sprite: Sprite {
                        color: kind.color(),
                        // slightly smaller to show borders
                        custom_size: Some(T::layout().square() - 0.25),
                        ..default()
                    },
                    transform: Transform::from_translation(
                        world_pos.extend(10.0),
                    ),
                    ..default()
                },
            );
        }
    }

    pub(super) fn change_square_kind<T: IntoMap>(
        mut cmd: Commands,
        mouse: Res<Input<MouseButton>>,
        mut map: ResMut<Map<T>>,

        squares: Query<Entity, With<SquareSprite>>,
        windows: Query<&Window, With<PrimaryWindow>>,
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

        squares.iter().for_each(|e| cmd.entity(e).despawn());

        spawn_grid(&mut cmd, &map);
    }

    pub(super) fn export_map<T: IntoMap>(map: Res<Map<T>>) {
        // for internal use only so who cares
        std::fs::write("map.ron", ron::to_string(&*map).unwrap()).unwrap();
    }

    impl SquareKind {
        const MAX_ZONE: u8 = 4;

        fn color(self) -> Color {
            let colors: [Color; Self::MAX_ZONE as usize + 1] = [
                Color::RED.with_a(0.5),
                Color::BLUE.with_a(0.5),
                Color::GREEN.with_a(0.5),
                Color::YELLOW.with_a(0.5),
                Color::PURPLE.with_a(0.5),
            ];

            match self {
                Self::None => Color::BLACK.with_a(0.25),
                Self::Wall => Color::BLACK.with_a(0.95),
                Self::Object => Color::WHITE.with_a(0.5),
                // if you want more zones, add more colors :-)
                Self::Zone(a) => colors[a as usize],
            }
        }

        fn next(self) -> Self {
            match self {
                Self::Object => Self::None,
                Self::None => Self::Wall,
                Self::Wall => Self::Zone(0),
                Self::Zone(Self::MAX_ZONE) => Self::Object,
                Self::Zone(a) => Self::Zone(a + 1),
            }
        }

        fn prev(self) -> Self {
            match self {
                Self::Object => Self::Zone(Self::MAX_ZONE),
                Self::None => Self::Object,
                Self::Wall => Self::None,
                Self::Zone(0) => Self::Wall,
                Self::Zone(a) => Self::Zone(a - 1),
            }
        }
    }
}
