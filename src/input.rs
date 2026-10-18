//! Read-only loading of the original xSAR deal and bounded manual solution CSV.
//!
//! Parsing creates compact typed actions and checks their structure and sequence.
//! It does not replay moves, infer current pile cards, or authorise guest input.

use std::{fs::File, io::Read, path::Path};

use crate::{
    parameters::{
        ConsoleData, InputError, Location, MAX_SOLUTION_BYTES, MAX_SOLUTION_RECORDS,
        PyramidPosition, RecordedAction, RecordedSolution, SolutionError, SolutionStep,
    },
    utility::location_from_row_column,
};


/// Open and validate all requested inputs before returning presentation data.
///
/// # Errors
/// Returns the input path and underlying open, xSAR codec or CSV validation error.
pub fn load_inputs(
    deal_path: &Path,
    solution_path: Option<&Path>,
) -> Result<ConsoleData, InputError> {
    let file = File::open(deal_path).map_err(|source| InputError::Open {
        path: deal_path.to_owned(),
        source,
    })?;
    let deal = xsar::pyramid_json::read_from(file).map_err(|source| InputError::Deal {
        path: deal_path.to_owned(),
        source,
    })?;
    let solution = solution_path
        .map(|path| {
            let file = File::open(path).map_err(|source| InputError::Open {
                path: path.to_owned(),
                source,
            })?;
            read_solution(file).map_err(|source| InputError::Solution {
                path: path.to_owned(),
                source,
            })
        })
        .transpose()?;

    let position = PyramidPosition::from_deal(&deal);
    Ok(ConsoleData {
        deal,
        position,
        solution,
    })
}


/// Decode a CSV recording within the console byte and action-count limits.
///
/// At most one byte beyond the byte cap is consumed to detect oversize input.
/// The retained boxed slice stores no CSV strings and has no spare Vec capacity.
///
/// # Errors
/// Refuses unreadable/oversized input, wrong headers, malformed records, invalid
/// locations, empty recordings and noncontiguous or duplicate sequence numbers.
pub fn read_solution(reader: impl Read) -> Result<RecordedSolution, SolutionError> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_SOLUTION_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_SOLUTION_BYTES {
        return Err(SolutionError::TooLarge);
    }

    let mut csv = csv::ReaderBuilder::new().from_reader(bytes.as_slice());
    if !csv
        .headers()?
        .iter()
        .eq(["SEQUENCE", "ACTION", "CARD1", "CARD2"])
    {
        return Err(SolutionError::Header);
    }

    let mut steps = Vec::new();
    let mut source_lines = Vec::new();
    let mut previous = 0;
    let mut reordered = false;
    for result in csv.records() {
        let record = result?;
        let line = record.position().map_or(1, csv::Position::line);
        if steps.len() >= MAX_SOLUTION_RECORDS {
            return Err(SolutionError::TooMany);
        }
        let step = parse_record(&record, line)?;
        reordered |= step.sequence < previous;
        previous = step.sequence;
        source_lines.push((step.sequence, line));
        steps.push(step);
    }
    if steps.is_empty() {
        return Err(SolutionError::Empty);
    }

    steps.sort_unstable_by_key(|step| step.sequence);
    source_lines.sort_unstable_by_key(|&(sequence, _)| sequence);
    for (ordinal, (step, &(_, line))) in steps.iter().zip(&source_lines).enumerate() {
        if usize::from(step.sequence) != ordinal + 1 {
            return Err(record_error(
                line,
                format!(
                    "expected unique contiguous SEQUENCE {}; found {} (duplicate or gap)",
                    ordinal + 1,
                    step.sequence
                ),
            ));
        }
    }

    Ok(RecordedSolution {
        steps: steps.into_boxed_slice(),
        reordered,
    })
}


/// Parse one four-field row, retaining MATCH selection order and exact actions.
///
/// # Errors
/// Rejects zero/nondecimal/out-of-range sequence numbers, unknown actions,
/// missing/forbidden fields, duplicate MATCH locations and invalid coordinates.
fn parse_record(record: &csv::StringRecord, line: u64) -> Result<SolutionStep, SolutionError> {
    let mut fields = record.iter();
    let (Some(sequence), Some(action), Some(first), Some(second), None) = (
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
    ) else {
        return Err(record_error(line, "expected exactly four CSV fields"));
    };
    let sequence = parse_decimal(sequence)
        .filter(|&value| (1..=MAX_SOLUTION_RECORDS).contains(&value))
        .and_then(|value| u16::try_from(value).ok())
        .ok_or_else(|| {
            record_error(
                line,
                format!("SEQUENCE must be an integer in 1..={MAX_SOLUTION_RECORDS}"),
            )
        })?;
    let action = match action {
        "MATCH" => {
            let first = parse_location(first, line)?;
            let second = parse_location(second, line)?;
            if first == second {
                return Err(record_error(
                    line,
                    "MATCH requires two distinct card locations",
                ));
            }
            RecordedAction::Match(first, second)
        }
        "KING" if second.is_empty() => RecordedAction::King(parse_location(first, line)?),
        "MOVE" if first.is_empty() && second.is_empty() => RecordedAction::Move,
        "RECYCLE" if first.is_empty() && second.is_empty() => RecordedAction::Recycle,
        "KING" => return Err(record_error(line, "KING requires CARD1 and an empty CARD2")),
        "MOVE" | "RECYCLE" => {
            return Err(record_error(
                line,
                "MOVE/RECYCLE require empty CARD1 and CARD2",
            ));
        }
        _ => {
            return Err(record_error(
                line,
                "ACTION must be MATCH, KING, MOVE or RECYCLE",
            ));
        }
    };

    Ok(SolutionStep { sequence, action })
}


/// Parse DL, DR or one-based fixed `row|column` without whitespace conversion.
///
/// # Errors
/// Returns a source-line error unless `1 <= column <= row <= 7` or a pile marker.
fn parse_location(text: &str, line: u64) -> Result<Location, SolutionError> {
    match text {
        "DL" => return Ok(Location::DL),
        "DR" => return Ok(Location::DR),
        _ => {}
    }
    let location = text.split_once('|').and_then(|(row, column)| {
        location_from_row_column(parse_decimal(row)?, parse_decimal(column)?)
    });

    location.ok_or_else(|| {
        record_error(
            line,
            "location must be DL, DR or row|column with 1 <= column <= row <= 7",
        )
    })
}


/// Decode nonempty ASCII decimal digits, refusing signs, spaces and overflow.
fn parse_decimal(text: &str) -> Option<usize> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}


/// Attach a bounded internal validation explanation to its source CSV line.
fn record_error(line: u64, reason: impl Into<String>) -> SolutionError {
    SolutionError::Record {
        line,
        reason: reason.into(),
    }
}
