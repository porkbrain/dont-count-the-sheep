use crate::Square;

pub struct ExactSizeSquareIterator<I> {
    count: usize,
    iter: I,
}

/// All bounds can be negative, but left <= right and bottom <= top
pub fn rectangle(
    [left, right, top, bottom]: [i32; 4],
) -> impl ExactSizeIterator<Item = Square> {
    assert!(left <= right, "Left ({left}) not <= right ({right})");
    assert!(bottom >= top, "Bottom ({bottom}) not >= top ({top})");

    // count must take into account negative indexes
    let count = (right - left + 1) * (bottom - top + 1);

    ExactSizeSquareIterator {
        iter: (top..=bottom)
            .flat_map(move |y| (left..=right).map(move |x| Square::new(x, y))),
        count: count as usize,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square;

    #[test]
    fn it_has_correct_size() {
        // 3 * 3

        let mut iter = rectangle([-1, 1, -1, 1]);

        assert_eq!(iter.len(), 9);

        assert_eq!(iter.next().unwrap(), square(-1, -1));
        assert_eq!(iter.next().unwrap(), square(0, -1));
        assert_eq!(iter.next().unwrap(), square(1, -1));
        assert_eq!(iter.next().unwrap(), square(-1, 0));
        assert_eq!(iter.next().unwrap(), square(0, 0));
        assert_eq!(iter.next().unwrap(), square(1, 0));
        assert_eq!(iter.next().unwrap(), square(-1, 1));
        assert_eq!(iter.next().unwrap(), square(0, 1));
        assert_eq!(iter.next().unwrap(), square(1, 1));

        assert!(iter.next().is_none());
    }

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
}
