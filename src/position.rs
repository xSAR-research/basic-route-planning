//! Mutable occupancy of fixed tableau positions and original stock slots.
//!
//! The validated complete deal remains the sole owner of card identities. This
//! model retains only occupancy, with one-based tableau coordinates and
//! zero-based original stock ordinals. An occupied tableau card is accessible
//! only when both positions directly below it are known empty; unknown occupants
//! still block. This is a topology and observation model, not a validator of
//! matching ranks, current pile tops, D-key moves, or recycle rules.

use xsar::games::pyramid::CompleteDeal;

use crate::parameters::{
    Occupancy, PositionError, PyramidPosition, STOCK_LEN, TABLEAU_LEN, TABLEAU_ROWS,
    TriangularMatrix,
};


impl PyramidPosition {
    /// Initialise occupied slots for a validated complete original deal.
    ///
    /// The borrowed deal supplies the completeness contract; no card values or
    /// references are retained or copied. All 28 tableau positions and 24 stock
    /// slots begin occupied. Stock ordinals preserve original encounter order
    /// and do not indicate which card is currently visible in DL or DR.
    pub fn from_deal(_deal: &CompleteDeal) -> Self {
        Self {
            tableau: TriangularMatrix::new([Occupancy::Occupied; TABLEAU_LEN]),
            stock: [Occupancy::Occupied; STOCK_LEN],
        }
    }


    /// Borrow the fixed tableau occupancy without permitting unchecked removal.
    ///
    /// Coordinates remain one-based and do not compact when positions empty.
    pub const fn tableau(&self) -> &TriangularMatrix<Occupancy> {
        &self.tableau
    }


    /// Borrow occupancy in original zero-based stock ordinal order, `0..24`.
    ///
    /// These slots describe observed original cards, not a changing stock or
    /// waste ordering. Manual encounter number is the original ordinal plus one.
    pub const fn stock(&self) -> &[Occupancy; STOCK_LEN] {
        &self.stock
    }


    /// Report whether an occupied tableau card is exposed by the fixed topology.
    ///
    /// Coordinates must satisfy `1 <= column <= row <= 7`. Empty and unknown
    /// positions return `false`. Occupied bottom-row cards return `true`; every
    /// other occupied card requires both `(row + 1, column)` and
    /// `(row + 1, column + 1)` to be known empty. An unknown child still blocks.
    /// This does not establish a legal King removal or sum-of-13 match.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::InvalidTableau`] for invalid coordinates,
    /// including extreme inputs, before any coordinate arithmetic occurs.
    pub fn tableau_accessible(&self, row: usize, column: usize) -> Result<bool, PositionError> {
        let occupant = self
            .tableau
            .get(row, column)
            .ok_or(PositionError::InvalidTableau { row, column })?;
        if *occupant != Occupancy::Occupied {
            return Ok(false);
        }
        if row == TABLEAU_ROWS {
            return Ok(true);
        }

        let left_empty = self.tableau.get(row + 1, column) == Some(&Occupancy::Empty);
        let right_empty = self.tableau.get(row + 1, column + 1) == Some(&Occupancy::Empty);
        Ok(left_empty && right_empty)
    }


    /// Mark an occupied, exposed tableau position as empty without shifting slots.
    ///
    /// Only this position changes. The immutable original deal and other card
    /// locations remain unchanged. The caller supplies evidence that removal
    /// occurred; this method checks topology rather than card-value legality.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::InvalidTableau`] for invalid one-based bounds,
    /// [`PositionError::TableauNotOccupied`] for empty or unknown positions, or
    /// [`PositionError::Covered`] unless both covering positions are known empty.
    /// Errors leave all occupancy unchanged.
    pub fn mark_tableau_empty(&mut self, row: usize, column: usize) -> Result<(), PositionError> {
        let occupant = self
            .tableau
            .get(row, column)
            .ok_or(PositionError::InvalidTableau { row, column })?;
        if *occupant != Occupancy::Occupied {
            return Err(PositionError::TableauNotOccupied { row, column });
        }
        if !self.tableau_accessible(row, column)? {
            return Err(PositionError::Covered { row, column });
        }

        let slot = self
            .tableau
            .get_mut(row, column)
            .ok_or(PositionError::InvalidTableau { row, column })?;
        *slot = Occupancy::Empty;
        Ok(())
    }


    /// Record uncertainty at one fixed tableau position, including a prior empty.
    ///
    /// Unknown occupancy conservatively blocks its parents and is not treated as
    /// evidence of removal. No other position or card identity changes.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::InvalidTableau`] for invalid one-based bounds,
    /// including extreme inputs. Errors leave all occupancy unchanged.
    pub fn mark_tableau_unknown(&mut self, row: usize, column: usize) -> Result<(), PositionError> {
        let slot = self
            .tableau
            .get_mut(row, column)
            .ok_or(PositionError::InvalidTableau { row, column })?;
        *slot = Occupancy::Unknown;
        Ok(())
    }


    /// Mark an occupied original stock slot empty using a zero-based ordinal.
    ///
    /// The caller supplies observed removal of that original card. This changes
    /// no slot order and checks neither current pile-top access nor recycle rules.
    /// Manual encounter number is `ordinal + 1` for valid ordinals `0..24`.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::InvalidStock`] for ordinals outside `0..24`,
    /// including extreme inputs, or [`PositionError::StockNotOccupied`] for an
    /// empty or unknown slot. Errors leave all occupancy unchanged.
    pub fn mark_stock_empty(&mut self, ordinal: usize) -> Result<(), PositionError> {
        let slot = self
            .stock
            .get_mut(ordinal)
            .ok_or(PositionError::InvalidStock { ordinal })?;
        if *slot != Occupancy::Occupied {
            return Err(PositionError::StockNotOccupied { ordinal });
        }

        *slot = Occupancy::Empty;
        Ok(())
    }


    /// Record uncertainty at one original stock slot using a zero-based ordinal.
    ///
    /// Any valid slot, including a prior empty slot, may become unknown. This
    /// neither removes a card nor infers a current DL/DR pile-top identity.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::InvalidStock`] for ordinals outside `0..24`,
    /// including extreme inputs. Errors leave all occupancy unchanged.
    pub fn mark_stock_unknown(&mut self, ordinal: usize) -> Result<(), PositionError> {
        let slot = self
            .stock
            .get_mut(ordinal)
            .ok_or(PositionError::InvalidStock { ordinal })?;
        *slot = Occupancy::Unknown;
        Ok(())
    }
}
