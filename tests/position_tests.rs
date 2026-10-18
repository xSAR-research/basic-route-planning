//! Public-consumer checks for independently owned, mutable Pyramid occupancy.
//!
//! The synthetic complete deal keeps every original identity while its position
//! records Occupied, Empty or Unknown. These checks cover exposure and mutation
//! bounds; they do not assert rank matching, stock/waste transitions or a route.

use std::{fs, path::PathBuf};

use basic_route_planning::{
    input::load_inputs,
    parameters::{ConsoleData, Occupancy, PositionError, PyramidPosition},
};


/// Locate this package's complete synthetic deck without a machine-specific path.
fn deal_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-synthetic-v2.json")
}


/// Read a complete original deal and its independently initialised position.
fn load_data() -> ConsoleData {
    load_inputs(&deal_path(), None).expect("load the complete synthetic deck")
}


/// The real input boundary creates 52 occupied slots with only the bottom exposed.
#[test]
fn loaded_deal_initialises_complete_occupied_position() {
    let data = load_data();

    assert_eq!(data.position, PyramidPosition::from_deal(&data.deal));
    assert_eq!(
        data.position.tableau().as_array(),
        &[Occupancy::Occupied; 28]
    );
    assert_eq!(data.position.stock(), &[Occupancy::Occupied; 24]);
    assert!(data.solution.is_none());

    for row in 1..=7 {
        for column in 1..=row {
            assert_eq!(
                data.position.tableau().get(row, column),
                Some(&Occupancy::Occupied)
            );
            assert_eq!(data.position.tableau_accessible(row, column), Ok(row == 7));
        }
    }
}


/// Either covering child blocks a parent; both empty children expose that parent.
#[test]
fn both_covering_children_must_be_empty_before_parent_removal() {
    let initial = load_data().position;
    assert_eq!(initial.tableau_accessible(6, 3), Ok(false));

    let mut left_only = initial.clone();
    left_only
        .mark_tableau_empty(7, 3)
        .expect("remove exposed left child");
    assert_eq!(left_only.tableau_accessible(6, 3), Ok(false));
    assert_eq!(left_only.tableau_accessible(7, 3), Ok(false));

    let mut right_only = initial.clone();
    right_only
        .mark_tableau_empty(7, 4)
        .expect("remove exposed right child");
    assert_eq!(right_only.tableau_accessible(6, 3), Ok(false));
    assert_eq!(right_only.tableau_accessible(7, 4), Ok(false));

    let mut both = left_only;
    both.mark_tableau_empty(7, 4)
        .expect("remove the other covering child");
    assert_eq!(both.tableau_accessible(6, 3), Ok(true));
    both.mark_tableau_empty(6, 3)
        .expect("remove the newly exposed parent");
    assert_eq!(both.tableau().get(6, 3), Some(&Occupancy::Empty));
    assert_eq!(both.tableau_accessible(6, 3), Ok(false));
    assert_eq!(both.tableau_accessible(6, 2), Ok(false));
    assert_eq!(both.tableau_accessible(6, 4), Ok(false));
}


/// A known empty neighbour cannot make an unknown covering child disappear.
#[test]
fn unknown_child_and_unknown_own_slot_are_not_accessible() {
    let mut position = load_data().position;
    position
        .mark_tableau_empty(7, 3)
        .expect("remove known exposed child");
    position
        .mark_tableau_unknown(7, 4)
        .expect("record uncertain other child");

    assert_eq!(position.tableau().get(7, 3), Some(&Occupancy::Empty));
    assert_eq!(position.tableau().get(7, 4), Some(&Occupancy::Unknown));
    assert_eq!(position.tableau_accessible(6, 3), Ok(false));
    assert_eq!(position.tableau_accessible(7, 4), Ok(false));

    let snapshot = position.clone();
    assert_eq!(
        position.mark_tableau_empty(6, 3),
        Err(PositionError::Covered { row: 6, column: 3 })
    );
    assert_eq!(position, snapshot);
    assert_eq!(
        position.mark_tableau_empty(7, 4),
        Err(PositionError::TableauNotOccupied { row: 7, column: 4 })
    );
    assert_eq!(position, snapshot);

    position
        .mark_tableau_unknown(7, 6)
        .expect("mark exposed own slot unknown");
    assert_eq!(position.tableau_accessible(7, 6), Ok(false));
}


/// Fresh uncertainty revokes exposure even after a child had been marked empty.
#[test]
fn unknown_annotation_revokes_previous_empty_child_evidence() {
    let mut position = load_data().position;
    position
        .mark_tableau_empty(7, 3)
        .expect("remove left child");
    position
        .mark_tableau_empty(7, 4)
        .expect("remove right child");
    assert_eq!(position.tableau_accessible(6, 3), Ok(true));

    position
        .mark_tableau_unknown(7, 3)
        .expect("record uncertainty about former empty slot");
    assert_eq!(position.tableau_accessible(6, 3), Ok(false));
    assert_eq!(position.tableau().get(7, 3), Some(&Occupancy::Unknown));
}


/// Covered and already removed cards cannot be removed by mutating the state first.
#[test]
fn refused_tableau_removals_preserve_the_entire_position() {
    let mut position = load_data().position;
    let occupied = position.clone();
    assert_eq!(
        position.mark_tableau_empty(6, 3),
        Err(PositionError::Covered { row: 6, column: 3 })
    );
    assert_eq!(position, occupied);

    position
        .mark_tableau_empty(7, 3)
        .expect("remove exposed bottom card");
    let removed = position.clone();
    assert_eq!(
        position.mark_tableau_empty(7, 3),
        Err(PositionError::TableauNotOccupied { row: 7, column: 3 })
    );
    assert_eq!(position, removed);
}


/// Invalid and extreme tableau coordinates are rejected before arithmetic or mutation.
#[test]
fn invalid_tableau_queries_and_mutations_are_atomic() {
    let mut position = load_data().position;
    for (row, column) in [
        (0, 0),
        (0, 1),
        (8, 1),
        (1, 0),
        (3, 4),
        (7, 8),
        (usize::MAX, 1),
        (1, usize::MAX),
        (usize::MAX, usize::MAX),
    ] {
        let snapshot = position.clone();
        assert_eq!(
            position.tableau_accessible(row, column),
            Err(PositionError::InvalidTableau { row, column })
        );
        assert_eq!(
            position.mark_tableau_empty(row, column),
            Err(PositionError::InvalidTableau { row, column })
        );
        assert_eq!(position, snapshot);
        assert_eq!(
            position.mark_tableau_unknown(row, column),
            Err(PositionError::InvalidTableau { row, column })
        );
        assert_eq!(position, snapshot);
        assert_eq!(position.tableau().get(row, column), None);
    }
}


/// Stock annotations use original zero-based ordinals without changing encounter order.
#[test]
fn stock_first_last_unknown_and_duplicate_removal_contracts() {
    let mut position = load_data().position;
    position
        .mark_stock_empty(0)
        .expect("mark first original stock card removed");
    position
        .mark_stock_empty(23)
        .expect("mark last original stock card removed");
    assert_eq!(position.stock()[0], Occupancy::Empty);
    assert_eq!(position.stock()[23], Occupancy::Empty);
    assert_eq!(&position.stock()[1..23], &[Occupancy::Occupied; 22]);

    let removed = position.clone();
    assert_eq!(
        position.mark_stock_empty(0),
        Err(PositionError::StockNotOccupied { ordinal: 0 })
    );
    assert_eq!(position, removed);
    assert_eq!(
        position.mark_stock_empty(23),
        Err(PositionError::StockNotOccupied { ordinal: 23 })
    );
    assert_eq!(position, removed);

    position
        .mark_stock_unknown(1)
        .expect("record uncertain original stock slot");
    assert_eq!(position.stock()[1], Occupancy::Unknown);
    let unknown = position.clone();
    assert_eq!(
        position.mark_stock_empty(1),
        Err(PositionError::StockNotOccupied { ordinal: 1 })
    );
    assert_eq!(position, unknown);

    position
        .mark_stock_unknown(0)
        .expect("record uncertainty about a former empty stock slot");
    assert_eq!(position.stock()[0], Occupancy::Unknown);
}


/// Stock bounds reject the first invalid ordinal and extreme input without changes.
#[test]
fn invalid_stock_annotations_preserve_the_entire_position() {
    let mut position = load_data().position;
    position
        .mark_stock_empty(0)
        .expect("establish a nontrivial position");
    for ordinal in [24, 25, usize::MAX] {
        let snapshot = position.clone();
        assert_eq!(
            position.mark_stock_empty(ordinal),
            Err(PositionError::InvalidStock { ordinal })
        );
        assert_eq!(position, snapshot);
        assert_eq!(
            position.mark_stock_unknown(ordinal),
            Err(PositionError::InvalidStock { ordinal })
        );
        assert_eq!(position, snapshot);
    }
}


/// Position changes retain every original identity, metadata field and input byte.
#[test]
fn position_mutation_preserves_original_deal_and_source_file() {
    let source = fs::read(deal_path()).expect("read original fixture bytes");
    let mut data = load_data();
    let original_deal = data.deal.clone();

    data.position
        .mark_tableau_empty(7, 3)
        .expect("remove left covering card");
    data.position
        .mark_tableau_empty(7, 4)
        .expect("remove right covering card");
    data.position
        .mark_tableau_empty(6, 3)
        .expect("remove exposed parent");
    data.position
        .mark_tableau_unknown(7, 6)
        .expect("retain uncertain tableau slot");
    data.position
        .mark_stock_empty(0)
        .expect("annotate first original stock slot");
    data.position
        .mark_stock_unknown(23)
        .expect("annotate last original stock slot");

    assert_eq!(data.deal, original_deal);
    assert_eq!(data.deal.tableau(), original_deal.tableau());
    assert_eq!(data.deal.stock(), original_deal.stock());
    assert_eq!(fs::read(deal_path()).expect("reread fixture bytes"), source);
    assert_ne!(data.position, PyramidPosition::from_deal(&original_deal));
}
