use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ItemStatus {
    Closed,
    InProgress,
    Open,
    Resolved,
}

impl Display for ItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "Closed"),
            Self::InProgress => write!(f, "IN PROGRESS"),
            Self::Open => write!(f, "OPEN"),
            Self::Resolved => write!(f, "RESOLVED"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    NavigateToEpicDetail { epic_id: u32 },
    NavigateToStoryDetail { epic_id: u32, story_id: u32 },
    NavigateToPreviousPage,
    CreateEpic,
    UpdateEpicStatus { epic_id: u32 },
    DeleteEpic { epic_id: u32 },
    CreateStory { epic_id: u32 },
    UpdateStoryStatus { story_id: u32 },
    DeleteStory { epic_id: u32, story_id: u32 },
    Exit,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ItemId(pub u32);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ItemDetail {
    pub description: String,
    pub id: ItemId,
    pub name: String,
    pub status: ItemStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Epic {
    pub detail: ItemDetail,
    pub stories: Vec<ItemId>,
}

impl Epic {
    pub fn new(detail: ItemDetail, stories: Vec<ItemId>) -> Self {
        return Epic { detail, stories };
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Story {
    pub detail: ItemDetail,
}

impl Story {
    pub fn new(detail: ItemDetail) -> Self {
        return Story { detail };
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum ItemType {
    Epic { id: ItemId },
    Story { id: ItemId },
    None,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DB {
    pub last_item: ItemType,
    pub epics: HashMap<u32, Epic>,
    pub stories: HashMap<u32, Story>,
}
