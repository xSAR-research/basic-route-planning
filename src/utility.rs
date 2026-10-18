//! Checked mapping between fixed tableau storage and manual recording locations.
//!
//! Recording rows increase from top to bottom and columns from left to right.
//! Both are one-based, while the sole stored array index remains zero-based.
//! Bounds are checked before arithmetic, including for extreme external inputs.

use crate::parameters::{Location, TABLEAU_LEN, TABLEAU_ROWS, TableauLocation};


/// Convert `1 <= column <= row <= 7` to its row-major index in `0..28`.
///
/// Invalid coordinates return `None` before multiplication or subtraction, so
/// even `usize::MAX` inputs cannot overflow or underflow the mapping formula.
pub fn tableau_index(row: usize, column: usize) -> Option<usize> {
    if !(1..=TABLEAU_ROWS).contains(&row) || !(1..=row).contains(&column) {
        return None;
    }

    Some(row * (row - 1) / 2 + (column - 1))
}


/// Convert a flat array index in `0..28` to its one-based row and column.
///
/// Returns `None` for every index outside the complete tableau. The bounded
/// seven-row scan uses integer arithmetic without floating-point rounding.
pub fn tableau_coordinates(index: usize) -> Option<(usize, usize)> {
    if index >= TABLEAU_LEN {
        return None;
    }

    let mut row_start = 0;
    for row in 1..=TABLEAU_ROWS {
        if index < row_start + row {
            return Some((row, index - row_start + 1));
        }
        row_start += row;
    }

    None
}


/// Construct a fixed tableau location from valid one-based row/column inputs.
///
/// Returns `None` unless `1 <= column <= row <= 7`; DRAW pile markers are never
/// produced by this coordinate-only conversion.
pub fn location_from_row_column(row: usize, column: usize) -> Option<Location> {
    tableau_index(row, column)
        .and_then(TableauLocation::from_index)
        .map(Location::Tableau)
}


/// Return a tableau location's one-based coordinates, or `None` for DL and DR.
pub fn row_column_from_location(location: Location) -> Option<(usize, usize)> {
    match location {
        Location::Tableau(location) => tableau_coordinates(location.index()),
        Location::DL | Location::DR => None,
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Exhaustively prove all 28 one-based coordinates round-trip in row order.
    #[test]
    fn every_tableau_coordinate_round_trips() {
        let mut expected_index = 0;
        let mut occurrences = [0; TABLEAU_LEN];
        for row in 1..=TABLEAU_ROWS {
            for column in 1..=row {
                let index = tableau_index(row, column).unwrap();
                assert_eq!(index, expected_index);
                *occurrences.get_mut(index).unwrap() += 1;
                assert_eq!(tableau_coordinates(expected_index), Some((row, column)));
                let location = location_from_row_column(row, column).unwrap();
                assert_eq!(row_column_from_location(location), Some((row, column)));
                expected_index += 1;
            }
        }

        assert_eq!(expected_index, TABLEAU_LEN);
        assert_eq!(occurrences, [1; TABLEAU_LEN]);
    }


    /// Reject invalid bounds before mapping arithmetic, including extreme inputs.
    #[test]
    fn invalid_and_extreme_coordinates_are_rejected() {
        for (row, column) in [
            (0, 0),
            (0, 1),
            (1, 0),
            (3, 4),
            (7, 8),
            (8, 1),
            (usize::MAX, 1),
            (1, usize::MAX),
            (usize::MAX, usize::MAX),
        ] {
            assert_eq!(tableau_index(row, column), None);
            assert_eq!(location_from_row_column(row, column), None);
        }

        assert_eq!(tableau_coordinates(TABLEAU_LEN), None);
        assert_eq!(tableau_coordinates(TABLEAU_LEN + 1), None);
        assert_eq!(tableau_coordinates(usize::MAX), None);
    }


    /// Pile locations cannot be mistaken for tableau coordinates.
    #[test]
    fn piles_have_no_tableau_coordinates() {
        assert_eq!(row_column_from_location(Location::DL), None);
        assert_eq!(row_column_from_location(Location::DR), None);
    }
}
