use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ItemStatus {
    Canceled,
    InProgress,
    Open,
    Resolved,
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
