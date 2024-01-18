use anyhow::{anyhow, Result};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use crate::model::*;

pub trait DataBase {
    fn read_db(&self) -> Result<DB>;
    fn write_db(&self, database: &DB) -> Result<()>;
}

pub struct JSONFileDatabase(PathBuf);

impl DataBase for JSONFileDatabase {
    fn read_db(&self) -> Result<DB> {
        let file = File::open(&self.0)?;
        let reader = BufReader::new(file);

        Ok(serde_json::from_reader(reader)?)
    }

    fn write_db(&self, database: &DB) -> Result<()> {
        let file = OpenOptions::new().write(true).truncate(true).open(&self.0)?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer_pretty(&mut writer, database)?;
        Ok(writer.flush()?)
    }
}

pub mod test_utils {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;

    pub struct MockDB {
        last_written_db: RefCell<DB>,
    }

    impl MockDB {
        pub fn new() -> Self {
            Self {
                last_written_db: RefCell::new(DB {
                    last_item: ItemType::None,
                    epics: HashMap::new(),
                    stories: HashMap::new(),
                }),
            }
        }
    }

    impl DataBase for MockDB {
        fn read_db(&self) -> Result<DB> {
            Ok(self.last_written_db.borrow().clone())
        }

        fn write_db(&self, database: &DB) -> Result<()> {
            *self.last_written_db.borrow_mut() = database.clone();
            Ok(())
        }
    }
}

pub struct JiraDataBase {
    pub database: Box<dyn DataBase>,
}

impl JiraDataBase {
    pub fn new(path: PathBuf) -> Self {
        return JiraDataBase {
            database: Box::new(JSONFileDatabase(path)),
        };
    }

    pub fn read_db(&self) -> Result<DB> {
        Ok(self.database.read_db()?)
    }

    pub fn create_epic(&self, name: String, description: String) -> Result<ItemId> {
        let mut db = self.database.read_db()?;
        let epic_id = match db.epics.keys().max() {
            None => ItemId(0),
            Some(last_id) => ItemId(last_id + 1),
        };
        let epic = Epic::new(
            ItemDetail {
                description,
                id: epic_id,
                name,
                status: ItemStatus::Open,
            },
            Vec::new(),
        );
        let epic_id = db.epics.entry(epic.detail.id.0).or_insert(epic).detail.id;

        db.last_item = ItemType::Epic {
            id: epic_id.clone(),
        };

        self.database.write_db(&db)?;
        Ok(epic_id)
    }

    pub fn create_story(
        &self,
        name: String,
        description: String,
        epic_id: Option<ItemId>,
    ) -> Result<ItemId> {
        let mut db = self.database.read_db()?;
        let story_id = match db.stories.keys().max() {
            None => ItemId(0),
            Some(last_id) => ItemId(last_id + 1),
        };
        let story = Story::new(ItemDetail {
            description,
            id: story_id,
            name,
            status: ItemStatus::Open,
        });
        let story_id = db
            .stories
            .entry(story.detail.id.0)
            .or_insert(story)
            .detail
            .id;

        db.last_item = ItemType::Story {
            id: story_id.clone(),
        };

        if let Some(id) = epic_id {
            let epic = db.epics.get(&id.0);

            match epic {
                None => return Err(anyhow!("Epic ID: {id:?} was not found")),
                Some(_) => db
                    .epics
                    .entry(id.0)
                    .and_modify(|epic| epic.stories.push(story_id.clone())),
            };
        }

        self.database.write_db(&db)?;
        Ok(story_id)
    }

    pub fn delete_epic(&self, id: ItemId) -> Result<()> {
        let mut db = self.database.read_db()?;

        if let ItemType::Epic { id: last_item_id } = db.last_item {
            if last_item_id.0 == id.0 {
                db.last_item = ItemType::None;
            }
        }

        if let Some(epic) = db.epics.get(&id.0) {
            for story_id in &epic.stories {
                self.delete_story(*story_id, None)?;
            }

            db = self.database.read_db()?;
        }

        return match db.epics.remove(&id.0) {
            Some(_) => Ok(self.database.write_db(&db)?),
            None => Err(anyhow!("Epic ID: {:?} was not found", id)),
        };
    }

    pub fn delete_story(&self, story_id: ItemId, epic_id: Option<ItemId>) -> Result<()> {
        let mut db = self.database.read_db()?;

        if let ItemType::Story { id: last_item_id } = db.last_item {
            if last_item_id.0 == story_id.0 {
                db.last_item = ItemType::None;
            }
        }

        if let Some(id) = epic_id {
            if !db.epics.contains_key(&id.0) {
                return Err(anyhow!("Epic ID: {id:?} was not found"));
            }

            db
                .epics
                .entry(id.0)
                .and_modify(|epic| {
                    epic.stories.remove(
                epic.stories
                        .iter()
                        .position(|id| id.0 == story_id.0)
                        .unwrap()
                    );
                });
        }

        return match db.stories.remove(&story_id.0) {
            Some(_) => Ok(self.database.write_db(&db)?),
            None => Err(anyhow!("Story ID: {:?} was not found.", story_id)),
        };
    }

    pub fn update_epic_status(&self, epic_id: ItemId, status: ItemStatus) -> Result<()> {
        let mut db = self.database.read_db()?;
        let epic = db.epics.get(&epic_id.0);

        match epic {
            Some(_) => {
                db.epics
                    .entry(epic_id.0)
                    .and_modify(|epic| epic.detail.status = status);
                Ok(self.database.write_db(&db)?)
            }
            None => Err(anyhow!("Epic ID: {:?} was not found.", epic_id)),
        }
    }

    pub fn update_story_status(&self, story_id: ItemId, status: ItemStatus) -> Result<()> {
        let mut db = self.database.read_db()?;
        let story = db.stories.get(&story_id.0);

        match story {
            Some(_) => {
                db.stories
                    .entry(story_id.0)
                    .and_modify(|story| story.detail.status = status);
                Ok(self.database.write_db(&db)?)
            }
            None => Err(anyhow!("Story ID: {:?} was not found.", story_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::MockDB;
    use super::*;

    #[test]
    fn create_epic_should_work() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let result = db.create_epic(
            "First Epic".to_owned(),
            "This is the first test epic".to_owned(),
        );

        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();
        let expected_id = ItemId(0);

        assert_eq!(id, expected_id);

        match db_state.last_item {
            ItemType::Epic { id } => assert_eq!(id, expected_id),
            _ => (),
        }

        let epic = db_state.epics.get(&id.0);

        assert!(epic.is_some());

        let epic = epic.unwrap();
        assert_eq!(epic.detail.name, "First Epic");
    }

    #[test]
    fn should_fail_when_creating_story_without_epic_id() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let result = db.create_story(
            "Failure Story Without Epic".to_owned(),
            "This story won't be created if the Epic ID is not valid nor found".to_owned(),
            Some(ItemId(90000)),
        );

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Epic ID: {:?} was not found", ItemId(90000))
        );
    }

    #[test]
    fn should_create_story_with_epic_id() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let epic_id = db
            .create_epic(
                "First Epic".to_owned(),
                "This is the first test epic".to_owned(),
            )
            .unwrap();
        let result = db.create_story(
            "Story With Epic".to_owned(),
            "This story will be part of an Epic".to_owned(),
            Some(epic_id),
        );

        assert!(result.is_ok());

        let data = db.database.read_db().unwrap();
        let epic = data.epics.get(&epic_id.0).unwrap();

        assert_eq!(epic.stories.len(), 1);
        assert_eq!(*epic.stories.first().unwrap(), result.unwrap())
    }

    #[test]
    fn should_create_story_without_epic_id() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let result = db.create_story(
            "First Story".to_owned(),
            "This is the first test story".to_owned(),
            None,
        );

        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();
        let expected_id = ItemId(0);

        assert_eq!(id, expected_id);

        match db_state.last_item {
            ItemType::Story { id } => assert_eq!(id, expected_id),
            _ => (),
        }

        let story = db_state.stories.get(&id.0);

        assert!(story.is_some());

        let story = story.unwrap();
        assert_eq!(story.detail.name, "First Story");
    }

    #[test]
    fn should_delete_an_epic() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let epic_id = db
            .create_epic(
                "First Epic".to_owned(),
                "This is the first test epic".to_owned(),
            )
            .unwrap();
        let result = db.delete_epic(epic_id);

        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_when_deleting_an_epic_with_invalid_id() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let result = db.delete_epic(ItemId(0));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Epic ID: {:?} was not found", ItemId(0))
        )
    }

    #[test]
    fn should_delete_story_without_epic_id() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let story_id = db
            .create_story(
                "First Story".to_owned(),
                "This is the first test story".to_owned(),
                None,
            )
            .unwrap();
        let result = db.delete_story(story_id, None);

        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_when_deleting_a_story_with_invalid_epic_id() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let epic_id = db
            .create_epic(
                "First Epic".to_owned(),
                "This is the first test epic".to_owned(),
            )
            .unwrap();
        let story_id = db
            .create_story(
                "Story With Epic".to_owned(),
                "This story will be part of an Epic".to_owned(),
                Some(epic_id),
            )
            .unwrap();
        let result = db.delete_story(story_id, Some(ItemId(90000)));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Epic ID: {:?} was not found", ItemId(90000))
        );
    }

    #[test]
    fn should_update_the_epic_status() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let epic_id = db
            .create_epic(
                "First Epic".to_owned(),
                "This is the first test epic".to_owned(),
            )
            .unwrap();
        let result = db.update_epic_status(epic_id, ItemStatus::Resolved);

        assert!(result.is_ok());

        let data = db.database.read_db().unwrap();
        let epic = data.epics.get(&epic_id.0).unwrap();

        assert_eq!(epic.detail.status, ItemStatus::Resolved);
    }

    #[test]
    fn should_fail_to_update_epic_with_invalid_id() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let result = db.update_epic_status(ItemId(0), ItemStatus::Closed);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Epic ID: {:?} was not found.", ItemId(0))
        );
    }

    #[test]
    fn should_update_story_status() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let story_id = db
            .create_story(
                "First Story".to_owned(),
                "This is the first test story".to_owned(),
                None,
            )
            .unwrap();
        let result = db.update_story_status(story_id, ItemStatus::Resolved);

        assert!(result.is_ok());

        let data = db.database.read_db().unwrap();
        let story = data.stories.get(&story_id.0).unwrap();

        assert_eq!(story.detail.status, ItemStatus::Resolved);
    }

    #[test]
    fn should_fail_to_update_story_status_with_invalid_id() {
        let db = JiraDataBase {
            database: Box::new(MockDB::new()),
        };
        let result = db.update_story_status(ItemId(0), ItemStatus::Closed);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Story ID: {:?} was not found.", ItemId(0))
        );
    }

    mod database {
        use std::{collections::HashMap, io::Write, path::Path};

        use super::*;

        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db = JSONFileDatabase(Path::new("INVALID_PATH").to_owned());
            assert!(db.read_db().is_err());
        }

        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let mut file = tempfile::NamedTempFile::new().unwrap();

            write!(
                file,
                r#"{{ "last_item": {{"type": "None" }}, epics: {{}}, "stories": {{}} }}"#
            )
            .unwrap();

            let db = JSONFileDatabase(file.path().to_owned());

            assert!(db.read_db().is_err());
        }

        #[test]
        fn read_db_should_parse_json_file() {
            let mut file = tempfile::NamedTempFile::new().unwrap();

            write!(
                file,
                r#"{{ "last_item": {{ "type": "None" }}, "epics": {{}}, "stories": {{}} }}"#
            )
            .unwrap();

            let db = JSONFileDatabase(file.path().to_owned());
            let result = db.read_db();

            assert!(result.is_ok());

            let result = result.unwrap();

            assert_eq!(result.epics.len(), 0);
            assert_eq!(result.stories.len(), 0);
        }

        #[test]
        fn write_db_should_work() {
            let mut file = tempfile::NamedTempFile::new().unwrap();

            write!(
                file,
                r#"{{ "last_item": {{ "type": "None" }}, "epics": {{}}, "stories": {{}} }}"#
            )
            .unwrap();

            let db = JSONFileDatabase(file.path().to_owned());

            let story = Story::new(ItemDetail {
                description: "New Test Story".to_owned(),
                id: ItemId(0),
                name: "New Test".to_owned(),
                status: ItemStatus::Open,
            });
            let epic = Epic::new(
                ItemDetail {
                    description: "New Test Epic".to_owned(),
                    id: ItemId(0),
                    name: "New Epic".to_owned(),
                    status: ItemStatus::Open,
                },
                vec![ItemId(story.detail.id.0)],
            );

            let mut stories = HashMap::new();
            let mut epics = HashMap::new();
            let last_item = ItemType::Epic {
                id: ItemId(epic.detail.id.0),
            };

            stories.insert(story.detail.id.0, story);
            epics.insert(epic.detail.id.0, epic);

            let state = DB {
                last_item,
                epics,
                stories,
            };
            let write_result = db.write_db(&state);
            let read_result = db.read_db().unwrap();

            assert_eq!(write_result.is_ok(), true);
            assert_eq!(read_result, state);
        }
    }
}
