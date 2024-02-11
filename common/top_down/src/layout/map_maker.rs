use std::collections::BTreeMap;

use bevy::{render::view::RenderLayers, utils::HashSet, window::PrimaryWindow};
use ron::ser::PrettyConfig;

use super::*;

#[derive(Component)]
pub(super) struct SquareSprite(Square);

#[derive(Resource, Reflect, InspectorOptions, Default)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct TileMapMakerToolbar<L: Tile> {
    // these are configurable
    // ~
    // ~
    /// What kind of tile to paint.
    paint: TileKind<L>,
    /// Each square has an associated list of tiles.
    /// Layer refers to the index in this list.
    /// We only manipulate the indexes of the tiles that equal to the
    /// `layer`.
    layer: usize,
    /// If set to true, will replace any tile kind.
    /// If set to false, will only paint over tiles that are `None`.
    paint_over_tiles: bool,

    // these are metadata used by the system
    // ~
    // ~
    /// We paint rectangles with this tool.
    /// When you click on a tile, it will start painting from there.
    /// When you release the mouse, it will stop painting and draw a
    /// rectangle of the `paint` kind from here to where you
    /// released the mouse.
    #[reflect(ignore)]
    begin_rect_at: Option<Square>,
    /// Copy of map is inserted when the map is loaded from fs, and then edited
    /// ONLY by the user map editing actions.
    /// No dynamic game logic should edit it.
    /// When we save the game, we store this map instead of the [`TileMap`]
    /// resource.
    ///
    /// We keep the copy in sync with the map resource in terms of the tiles
    /// being laid out in the same layers.
    #[reflect(ignore)]
    copy_of_map: HashMap<Square, SmallVec<[TileKind<L>; 3]>>,
}

pub(super) fn visualize_map<T: IntoMap>(
    mut cmd: Commands,
    map: Res<TileMap<T>>,
) {
    let root = cmd
        .spawn((
            Name::new("Debug Layout Grid"),
            SpatialBundle {
                transform: Transform::from_translation(Vec2::ZERO.extend(10.0)),
                ..default()
            },
        ))
        .id();

    for square in bevy_grid_squared::shapes::rectangle(T::bounds()) {
        let world_pos = T::layout().square_to_world_pos(square);

        let kind = map
            .squares
            .get(&square)
            .and_then(|tiles| tiles.first()) // default to first layer
            .copied()
            .unwrap_or(TileKind::Empty);

        let child = cmd
            .spawn((SquareSprite(square), Name::new(format!("{square}"))))
            .insert(SpriteBundle {
                sprite: Sprite {
                    color: kind.color(),
                    // slightly smaller to show borders
                    custom_size: Some(T::layout().square() - 0.25),
                    ..default()
                },
                transform: Transform::from_translation(world_pos.extend(0.0)),
                ..default()
            })
            .id();
        cmd.entity(root).add_child(child);
    }
}

pub(super) fn change_square_kind<T: IntoMap>(
    mouse: Res<Input<MouseButton>>,
    mut map: ResMut<TileMap<T>>,
    mut toolbar: ResMut<TileMapMakerToolbar<T::LocalTileKind>>,
    keyboard: Res<Input<KeyCode>>,

    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&RenderLayers>)>,
) {
    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft);
    let just_pressed_left = mouse.just_pressed(MouseButton::Left);
    let just_released_left = mouse.just_released(MouseButton::Left);
    let just_pressed_right = mouse.just_pressed(MouseButton::Right);

    // a) hold ctrl + press left to paint rect
    let start_painting_rect =
        ctrl_pressed && just_pressed_left && toolbar.begin_rect_at.is_none();
    // b) if painting rect, release left to stop painting
    let stop_painting_rect =
        toolbar.begin_rect_at.is_some() && just_released_left;
    // c) press right to paint single square
    let paint_single_square = just_pressed_right;

    // if neither of these, then early return
    if !start_painting_rect && !stop_painting_rect && !paint_single_square {
        return;
    }

    let Some(clicked_at) = cursor_to_square(T::layout(), windows, cameras)
    else {
        return;
    };

    if start_painting_rect {
        toolbar.begin_rect_at = Some(clicked_at);
    } else if stop_painting_rect
        && let Some(begin_rect_at) = toolbar.begin_rect_at.take()
    {
        for square in selection_rect(begin_rect_at, clicked_at) {
            try_paint(&mut toolbar, &mut map, square);
        }
    } else if paint_single_square {
        try_paint(&mut toolbar, &mut map, clicked_at);
    }
}

/// If a square can be painted, paint it.
fn try_paint<T: IntoMap>(
    toolbar: &mut TileMapMakerToolbar<T::LocalTileKind>,
    map: &mut TileMap<T>,
    at: Square,
) {
    if !T::contains(at) {
        return;
    }

    let tiles = map.squares.entry(at).or_default();
    if tiles.len() <= toolbar.layer {
        tiles.resize(toolbar.layer + 1, TileKind::Empty);
    }

    let can_paint =
        toolbar.paint_over_tiles || tiles[toolbar.layer] == TileKind::Empty;
    if !can_paint {
        return;
    }

    tiles[toolbar.layer] = toolbar.paint;
    // store the user change to the copy that will be saved
    toolbar.copy_of_map.insert(at, tiles.clone());
}

pub(super) fn recolor_squares<T: IntoMap>(
    map: ResMut<TileMap<T>>,
    toolbar: Res<TileMapMakerToolbar<T::LocalTileKind>>,

    mut squares: Query<(&SquareSprite, &mut Sprite)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&RenderLayers>)>,
) {
    let squares_painted: Option<HashSet<_>> =
        toolbar.begin_rect_at.and_then(|begin_rect_at| {
            let clicked_at = cursor_to_square(T::layout(), windows, cameras)?;

            Some(selection_rect(begin_rect_at, clicked_at).collect())
        });

    for (SquareSprite(at), mut sprite) in squares.iter_mut() {
        let tile_kind = map
            .squares
            .get(at)
            .and_then(|tiles| tiles.get(toolbar.layer))
            .copied()
            .unwrap_or_default();

        // show where we're painting unless we're not allowed to
        // paint over tiles
        let color =
            if squares_painted.as_ref().map_or(false, |s| s.contains(at))
                && (toolbar.paint_over_tiles || tile_kind == TileKind::Empty)
            {
                toolbar.paint.color()
            } else {
                tile_kind.color()
            };

        sprite.color = color;
    }
}

pub(super) fn export_map<T: IntoMap>(
    toolbar: Res<TileMapMakerToolbar<T::LocalTileKind>>,
) {
    // equivalent to tile map, but sorted so that we can serialize it
    // and the output is deterministic
    //
    // this struct MUST serialize to a compatible ron output as TileMap
    #[derive(Serialize)]
    struct SortedTileMap<T: IntoMap> {
        squares: BTreeMap<Square, SmallVec<[TileKind<T::LocalTileKind>; 3]>>,
        #[serde(skip)]
        _phantom: PhantomData<T>,
    }

    let mut tilemap_but_sorted: SortedTileMap<T> = SortedTileMap {
        squares: toolbar.copy_of_map.clone().into_iter().collect(),
        _phantom: default(),
    };

    // filter out needless squares
    tilemap_but_sorted.squares.retain(|sq, tiles| {
        if !T::contains(*sq) {
            return false;
        }

        for tile in tiles.as_slice() {
            match tile {
                // Should not happen
                TileKind::Actor(_) => {
                    error!("Actor tile found in toolbar map");
                    return false;
                }
                // these are fine
                TileKind::Wall | TileKind::Empty | TileKind::Trail => {}
                // fine for now but we might want to skip some of these in the
                // future
                TileKind::Local(_) => {}
            }
        }

        while tiles.ends_with(&[TileKind::Empty]) {
            tiles.pop();
        }

        !tiles.is_empty()
    });

    // for internal use only so who cares about unwraps and paths
    std::fs::write(
        format!("main_game/assets/{}", T::asset_path()),
        ron::ser::to_string_pretty(
            &tilemap_but_sorted,
            PrettyConfig::default()
                .compact_arrays(true)
                .separate_tuple_members(false)
                .indentor(" ".to_string())
                .depth_limit(2),
        )
        .unwrap(),
    )
    .unwrap();
}

impl<L> TileKind<L> {
    fn color(self) -> Color {
        match self {
            Self::Empty => Color::BLACK.with_a(0.25),
            Self::Wall => Color::BLACK.with_a(0.8),
            Self::Trail => Color::WHITE.with_a(0.25),
            Self::Actor { .. } => Color::GOLD.with_a(0.25),
            Self::Local(_) => Color::RED.with_a(0.25),
        }
    }
}

fn selection_rect(
    begin_rect_at: Square,
    clicked_at: Square,
) -> impl ExactSizeIterator<Item = Square> {
    let left = begin_rect_at.x.min(clicked_at.x);
    let right = begin_rect_at.x.max(clicked_at.x);
    let bottom = begin_rect_at.y.min(clicked_at.y);
    let top = begin_rect_at.y.max(clicked_at.y);

    bevy_grid_squared::shapes::rectangle([left, right, bottom, top])
}

fn cursor_to_square(
    layout: &SquareLayout,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&RenderLayers>)>,
) -> Option<Square> {
    let cursor_pos = windows.single().cursor_position()?;

    let (camera, camera_transform, _) = cameras
        .iter()
        .filter(|(_, _, l)| {
            l.map(|l| l.intersects(&RenderLayers::layer(0)))
                .unwrap_or(true)
        })
        .next()?;
    let world_pos =
        camera.viewport_to_world_2d(camera_transform, cursor_pos)?;

    Some(layout.world_pos_to_square(world_pos))
}

impl<L: Tile> TileMapMakerToolbar<L> {
    pub(super) fn new(
        copy_of_map: HashMap<Square, SmallVec<[TileKind<L>; 3]>>,
    ) -> Self {
        Self {
            copy_of_map,
            paint: default(),
            layer: 0,
            paint_over_tiles: false,
            begin_rect_at: None,
        }
    }
}