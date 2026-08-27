use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub notes: i64,
    pub cards: i64,
    pub media: i64,
    pub decks: i64,
    pub warnings: Vec<String>,
    pub root_deck_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckSummary {
    pub deck: Deck,
    pub due: i64,
    pub news: i64,
    pub total: i64,
    pub children: Vec<DeckSummary>,
}

impl DeckSummary {
    pub fn due_tree(&self) -> i64 {
        self.due + self.children.iter().map(Self::due_tree).sum::<i64>()
    }

    pub fn new_tree(&self) -> i64 {
        self.news + self.children.iter().map(Self::new_tree).sum::<i64>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayCounts {
    pub due: i64,
    pub news: i64,
}

impl TodayCounts {
    pub fn total(&self) -> i64 {
        self.due + self.news
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyCard {
    pub card_id: i64,
    pub note_id: i64,
    pub deck_name: String,
    pub tags: String,
    pub front: String,
    pub back: String,
    pub front_html: String,
    pub back_html: String,
    pub images: Vec<String>,
    pub audio: Vec<String>,
    pub video: Vec<String>,
    pub state: String,
    pub intervals: Intervals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervals {
    pub again: String,
    pub hard: String,
    pub good: String,
    pub easy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseRow {
    pub card_id: i64,
    pub note_id: i64,
    pub front: String,
    pub back: String,
    pub state: String,
    pub due: DateTime<Utc>,
    pub suspended: bool,
    pub tags: String,
}

#[derive(Debug, Clone)]
pub struct CardRow {
    pub id: i64,
    pub note_id: i64,
    pub deck_id: i64,
    pub template_id: i64,
    pub ordinal: i64,
    pub state: String,
    pub due: DateTime<Utc>,
    pub stability: Option<f64>,
    pub difficulty: Option<f64>,
    pub reps: i64,
    pub lapses: i64,
    pub scheduled_days: i64,
    pub last_review: Option<DateTime<Utc>>,
    pub first_reviewed_at: Option<DateTime<Utc>>,
    pub suspended: bool,
}
