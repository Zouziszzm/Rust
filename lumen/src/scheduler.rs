use crate::models::{CardRow, Intervals};
use chrono::{DateTime, Duration, Utc};
use rs_fsrs::{Card, Rating, State, FSRS};

pub fn rate_card(row: &CardRow, rating: u8, now: DateTime<Utc>) -> Card {
    let fsrs = FSRS::default();
    let current = to_fsrs(row, now);
    let rating = to_rating(rating);
    fsrs.next(current, now, rating).card
}

pub fn preview_intervals(row: &CardRow, now: DateTime<Utc>) -> Intervals {
    let fsrs = FSRS::default();
    let current = to_fsrs(row, now);
    let log = fsrs.repeat(current, now);
    Intervals {
        again: fmt_interval(log[&Rating::Again].card.due, now),
        hard: fmt_interval(log[&Rating::Hard].card.due, now),
        good: fmt_interval(log[&Rating::Good].card.due, now),
        easy: fmt_interval(log[&Rating::Easy].card.due, now),
    }
}

pub fn state_name(state: State) -> &'static str {
    match state {
        State::New => "new",
        State::Learning => "learning",
        State::Review => "review",
        State::Relearning => "relearning",
    }
}

fn to_rating(rating: u8) -> Rating {
    match rating {
        1 => Rating::Again,
        2 => Rating::Hard,
        3 => Rating::Good,
        _ => Rating::Easy,
    }
}

fn to_fsrs(row: &CardRow, now: DateTime<Utc>) -> Card {
    if row.state == "new" || row.reps == 0 {
        return Card::default();
    }
    Card {
        due: row.due,
        stability: row.stability.unwrap_or(0.0),
        difficulty: row.difficulty.unwrap_or(0.0),
        elapsed_days: row
            .last_review
            .map(|last| (now - last).num_days())
            .unwrap_or(0),
        scheduled_days: row.scheduled_days,
        reps: row.reps as i32,
        lapses: row.lapses as i32,
        state: match row.state.as_str() {
            "learning" => State::Learning,
            "relearning" => State::Relearning,
            "review" => State::Review,
            _ => State::New,
        },
        last_review: row.last_review.unwrap_or(now),
    }
}

fn fmt_interval(due: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let delta = due.signed_duration_since(now);
    if delta < Duration::minutes(10) {
        "<10m".into()
    } else if delta < Duration::hours(1) {
        format!("{}m", delta.num_minutes())
    } else if delta < Duration::hours(24) {
        format!("{}h", delta.num_hours())
    } else {
        format!("{}d", delta.num_days().max(1))
    }
}
