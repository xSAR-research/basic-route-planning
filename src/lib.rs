//! Bounded saved-deal intake and an independently testable Pyramid position model.
//!
//! The validated original deal owns card identities. Separate fixed occupancy
//! arrays record known, unknown and empty positions without changing those cards.
//! Accessibility checks describe tableau covering geometry; they do not validate
//! complete game moves, stock/waste transitions or authorise guest input.

pub mod input;
pub mod parameters;
pub mod position;
pub mod utility;


/// Pure complete-deal transitions under the pinned upstream rules.
pub mod rules;


/// Independent explicit-pile route validation.
pub mod replay;


/// Bounded minimum-cost A* and reference Dijkstra search.
pub mod search;
