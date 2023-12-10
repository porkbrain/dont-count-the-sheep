use std::marker::PhantomData;

use bevy::{math::vec2, prelude::*};

#[derive(Event)]
pub struct PoissonsEquationUpdateEvent<T> {
    /// The position within the field
    pub(crate) coords: GridCoords,
    /// If there's already a source at the position, this is added to it.
    /// Otherwise, it's set as the source.
    pub(crate) delta: f32,
    phantom: PhantomData<T>,
}

pub(crate) type Grid = Vec<Vec<GridPoint>>;

#[derive(Resource)]
pub struct PoissonsEquation<T> {
    /// Optimization which allows us to reach convergence usually faster if
    /// the factor is picked well.
    pub overcorrection_factor: f32,
    /// We keep track of the total difference between the old and new
    /// values after each iteration.
    /// If it's close to zero, we stop iterating.
    pub last_smoothing_correction: f32,
    /// We set this to false if correction small and getting smaller.
    /// It's set to false on event received.
    pub stop_smoothing_out: bool,
    pub(crate) width: usize,
    pub(crate) height: usize,
    grid: Grid,
    phantom: PhantomData<T>,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum GridPoint {
    Average(f32),
    Source(f32),
}

/// It's your responsibility to make sure that the coordinates are within
/// the field.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GridCoords {
    pub x: usize,
    pub y: usize,
}

#[cfg(feature = "poissons-eq-visualization")]
#[derive(Component)]
pub struct VectorArrow;

#[cfg(feature = "poissons-eq-visualization")]
pub trait WorldDimensions {
    fn width() -> f32;
    fn height() -> f32;
}

impl<T: Send + Sync + 'static> PoissonsEquationUpdateEvent<T> {
    pub fn send<P: Into<GridCoords>>(
        update: &mut EventWriter<Self>,
        delta: f32,
        world_pos: P,
    ) -> GridCoords {
        let coords = world_pos.into();
        update.send(Self {
            delta,
            coords,
            phantom: PhantomData,
        });

        coords
    }
}

impl<T> PoissonsEquation<T> {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 2 && height > 2, "Field must be at least 3x3");

        Self {
            overcorrection_factor: 1.0,
            stop_smoothing_out: false, // force at least one iteration
            last_smoothing_correction: 0.0,
            width,
            height,
            grid: vec![vec![GridPoint::Average(0.0); width]; height],
            phantom: PhantomData,
        }
    }

    /// Sets the top row to be repulsive and the bottom row to be attractive.
    pub fn with_downward_attraction(mut self) -> Self {
        self.grid[0] = vec![GridPoint::Source(0.0); self.width];
        self.grid[self.height - 1] = vec![GridPoint::Source(1.0); self.width];

        self
    }

    /// Must be more than zero
    pub fn with_overcorrection_factor(mut self, factor: f32) -> Self {
        assert!(self.overcorrection_factor > 0.0);

        self.overcorrection_factor = factor;
        self
    }

    pub fn with_initial_smoothing(mut self, iterations: usize) -> Self {
        if iterations == 0 {
            return self;
        }

        for _ in 1..iterations {
            self.smooth_out();
        }

        self.last_smoothing_correction = self.smooth_out();

        debug!(
            "Initial smoothing with {iterations} ended up with correction {}",
            self.last_smoothing_correction
        );

        self
    }

    pub fn gradient_at<P: Into<GridCoords>>(&self, world_pos: P) -> Vec2 {
        let GridCoords { x, y } = world_pos.into();
        let at: f32 = self.grid[y][x].into();

        // look at value to the right to find the x coordinate
        let gradient_x = self.grid[y]
            .get(x + 1)
            .map(|val| val.inner() - at)
            // as if the last value continued on forever
            .unwrap_or_else(|| self.grid[y][x - 1].inner() - at);

        // look at value above to find the y coordinate
        let gradient_y = self
            .grid
            .get(y + 1)
            .map(|row| at - row[x].inner())
            // as if the last value continued on forever
            .unwrap_or_else(|| self.grid[y - 1][x].inner() - at);

        vec2(gradient_x, gradient_y)
    }

    pub(crate) fn set(&mut self, coords: GridCoords, value: f32) {
        let GridCoords { x, y } = coords;

        match self.grid[y][x] {
            GridPoint::Average(_) => {
                self.grid[y][x] = GridPoint::Source(value);
            }
            GridPoint::Source(existing) => {
                self.grid[y][x] = GridPoint::Source(existing + value);
            }
        }
    }

    /// In [`Self::new`] we assert that self.width > 2 && self.height > 2
    pub(crate) fn smooth_out(&mut self) -> f32 {
        let mut total_correction = 0.0;

        for y in 0..self.height {
            for x in 0..self.width {
                if matches!(self.grid[y][x], GridPoint::Source(_)) {
                    continue;
                }

                let old_value = self.grid[y][x].inner();

                let above = if y == 0 {
                    old_value
                } else {
                    self.grid[y - 1][x].inner()
                };
                let below = self
                    .grid
                    .get(y + 1)
                    .map(|r| r[x].inner())
                    .unwrap_or(old_value);

                let left = if x == 0 {
                    old_value
                } else {
                    self.grid[y][x - 1].inner()
                };
                let right =
                    self.grid[y].get(x + 1).map(f32::from).unwrap_or(old_value);

                let sum = above + below + left + right;

                let delta = (sum - 4.0 * old_value) / 4.0;
                total_correction = delta.abs();

                self.grid[y][x] = GridPoint::Average(
                    old_value + delta * self.overcorrection_factor,
                );
            }
        }

        total_correction
    }
}

impl GridPoint {
    #[inline]
    fn inner(self) -> f32 {
        match self {
            GridPoint::Average(average) => average,
            GridPoint::Source(source) => source,
        }
    }
}

impl From<GridPoint> for f32 {
    #[inline]
    fn from(point: GridPoint) -> Self {
        point.inner()
    }
}

impl From<&GridPoint> for f32 {
    #[inline]
    fn from(point: &GridPoint) -> Self {
        (*point).inner()
    }
}
