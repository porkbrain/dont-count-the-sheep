use crate::Square;

pub struct ExactSizeSquareIterator<I> {
    count: usize,
    iter: I,
}

pub fn rectangle_between(
    from: Square,
    to: Square,
) -> impl ExactSizeIterator<Item = Square> {
    let left = from.x.min(to.x);
    let right = from.x.max(to.x);
    let bottom = from.y.min(to.y);
    let top = from.y.max(to.y);

    rectangle([left, right, bottom, top])
}

/// All bounds can be negative, but left <= right and bottom <= top
pub fn rectangle(
    [left, right, bottom, top]: [i32; 4],
) -> impl ExactSizeIterator<Item = Square> {
    assert!(left <= right, "Left ({left}) not <= right ({right})");
    assert!(top >= bottom, "Top ({top}) not >= bottom ({bottom})");

    // count must take into account negative indexes
    let count = (right - left + 1) * (top - bottom + 1);

    ExactSizeSquareIterator {
        iter: (bottom..=top)
            .flat_map(move |y| (left..=right).map(move |x| Square::new(x, y))),
        count: count as usize,
    }
}

/// An implementation of [Bresenham's circle algorithm].
///
/// This uses four quadrants, so calling `next()` will return a point for
/// the first quadrant, then the second, third, fourth and then back to
/// first.
///
/// [Bresenham's circle algorithm]: http://members.chello.at/~easyfilter/bresenham.html
pub fn bresenham_circle(
    center: Square,
    radius: i32,
) -> impl Iterator<Item = Square> {
    bresenham_circle::BresenhamCircle::new(center, radius)
}

impl<I> Iterator for ExactSizeSquareIterator<I>
where
    I: Iterator<Item = Square>,
{
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.count = self.count.saturating_sub(1);
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<I> ExactSizeIterator for ExactSizeSquareIterator<I> where
    I: Iterator<Item = Square>
{
}

mod bresenham_circle {
    //! This code was copied from <https://github.com/expenses/line_drawing>

    use crate::Square;

    pub(super) struct BresenhamCircle {
        x: i32,
        y: i32,
        center: Square,
        radius: i32,
        error: i32,
        quadrant: u8,
    }

    impl BresenhamCircle {
        #[inline]
        pub(super) fn new(center: Square, radius: i32) -> Self {
            Self {
                center,
                radius,
                x: -radius,
                y: 0,
                error: 2 - 2 * radius,
                quadrant: 1,
            }
        }
    }

    impl Iterator for BresenhamCircle {
        type Item = Square;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.x >= 0 {
                return None;
            }

            let (next_x, next_y) = match self.quadrant {
                1 => (self.center.x - self.x, self.center.y + self.y),
                2 => (self.center.x - self.y, self.center.y - self.x),
                3 => (self.center.x + self.x, self.center.y - self.y),
                4 => (self.center.x + self.y, self.center.y + self.x),
                _ => unreachable!(),
            };

            // Update the variables after each set of quadrants
            if self.quadrant == 4 {
                self.radius = self.error;

                if self.radius <= self.y {
                    self.y += 1;
                    self.error += self.y * 2 + 1;
                }

                if self.radius > self.x || self.error > self.y {
                    self.x += 1;
                    self.error += self.x * 2 + 1;
                }
            }

            self.quadrant = self.quadrant % 4 + 1;

            Some(Square::new(next_x, next_y))
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::utils::hashbrown::HashSet;

    use super::*;
    use crate::sq;

    #[test]
    fn it_creates_rect() {
        for left in -20..=20 {
            for right in 0..=20 {
                for top in -20..=20 {
                    for bottom in 0..=20 {
                        let mut iter =
                            rectangle([left, left + right, top, top + bottom]);
                        let len = iter.len();

                        let first = iter.next().unwrap();
                        let mut total = 1;
                        let mut prev = first;
                        for square in iter {
                            // we go from top left corner (so x increases) to
                            // the bottom right corner (so y decreases)
                            assert!(
                                square.x > prev.x || square.y > prev.y,
                                "{square:?} prev: {prev:?}",
                            );

                            total += 1;
                            prev = square;
                        }

                        assert_eq!(len, total);
                    }
                }
            }
        }
    }

    #[test]
    fn rect_has_correct_size() {
        // 3 * 3

        let mut iter = rectangle([-1, 1, -1, 1]);

        assert_eq!(iter.len(), 9);

        assert_eq!(iter.next().unwrap(), sq(-1, -1));
        assert_eq!(iter.next().unwrap(), sq(0, -1));
        assert_eq!(iter.next().unwrap(), sq(1, -1));
        assert_eq!(iter.next().unwrap(), sq(-1, 0));
        assert_eq!(iter.next().unwrap(), sq(0, 0));
        assert_eq!(iter.next().unwrap(), sq(1, 0));
        assert_eq!(iter.next().unwrap(), sq(-1, 1));
        assert_eq!(iter.next().unwrap(), sq(0, 1));
        assert_eq!(iter.next().unwrap(), sq(1, 1));

        assert!(iter.next().is_none());
    }

    #[test]
    fn bresenham_circle_contains_rim_of_circle_with_radius_3() {
        let rim: HashSet<_> = vec![
            Square { x: 3, y: 0 },
            Square { x: 0, y: 3 },
            Square { x: -3, y: 0 },
            Square { x: 0, y: -3 },
            Square { x: 3, y: 1 },
            Square { x: -1, y: 3 },
            Square { x: -3, y: -1 },
            Square { x: 1, y: -3 },
            Square { x: 2, y: 2 },
            Square { x: -2, y: 2 },
            Square { x: -2, y: -2 },
            Square { x: 2, y: -2 },
            Square { x: 1, y: 3 },
            Square { x: -3, y: 1 },
            Square { x: -1, y: -3 },
            Square { x: 3, y: -1 },
        ]
        .into_iter()
        .collect();

        for square in bresenham_circle(sq(0, 0), 3) {
            assert!(rim.contains(&square));
        }
    }
}
