use anyhow::{anyhow, Context, Ok, Result};
use std::rc::Rc;

use crate::db::JiraDataBase;
use crate::model::{Action, ItemId};
use crate::ui::{EpicDetail, HomePage, Page, Prompts, StoryDetail};

pub struct Navigator {
    pages: Vec<Box<dyn Page>>,
    prompts: Prompts,
    database: Rc<JiraDataBase>,
}

impl Navigator {
    pub fn new(database: Rc<JiraDataBase>) -> Self {
        Self {
            pages: vec![Box::new(HomePage {
                db: Rc::clone(&database),
            })],
            prompts: Prompts::new(),
            database,
        }
    }

    pub fn get_current_page(&self) -> Option<&Box<dyn Page>> {
        return self.pages.last();
    }

    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::CreateEpic => {
                let epic = (self.prompts.create_epic)();

                self.database
                    .create_epic(epic.detail.name, epic.detail.description)
                    .with_context(|| anyhow!("Failed to create Epic"))?;
            }
            Action::DeleteEpic { epic_id } => {
                if (self.prompts.delete_epic)() {
                    self.database
                        .delete_epic(ItemId(epic_id))
                        .with_context(|| anyhow!("Failed to delete epic"))?;
                }

                if !self.pages.is_empty() {
                    self.pages.pop();
                }
            }
            Action::Exit => self.pages.clear(),
            Action::CreateStory { epic_id } => {
                let story = (self.prompts.create_story)();

                self.database
                    .create_story(
                        story.detail.name,
                        story.detail.description,
                        Some(ItemId(epic_id)),
                    )
                    .with_context(|| anyhow!("Failed to create story"))?;
            },
            Action::DeleteStory { epic_id, story_id } => {
              if (self.prompts.delete_story)() {
                self
                  .database
                  .delete_story(ItemId(story_id), Some(ItemId(epic_id)))
                  .with_context(|| anyhow!("Failed to delete story"))?;
              }

              if !self.pages.is_empty() {
                self.pages.pop();
              }
            },
            Action::NavigateToEpicDetail { epic_id } => {
              self.pages.push(Box::new(EpicDetail { epic_id, db: Rc::clone(&self.database) }));
            },
            Action::NavigateToPreviousPage => {
              if !self.pages.is_empty() {
                self.pages.pop();
              }
            },
            Action::NavigateToStoryDetail { epic_id, story_id } => {
              self.pages.push(Box::new(StoryDetail { epic_id, story_id, db: Rc::clone(&self.database) }))
            },
            Action::UpdateEpicStatus { epic_id } => {
              if let Some(status) = (self.prompts.update_status)() {
                self
                  .database
                  .update_epic_status(ItemId(epic_id), status)
                  .with_context(|| anyhow!("Failed to update epic status"))?;
              }
            },
            Action::UpdateStoryStatus { story_id } => {
              if let Some(status) = (self.prompts.update_status)() {
                self
                  .database
                  .update_story_status(ItemId(story_id), status)
                  .with_context(|| anyhow!("Failed to update story status"))?;
              }
            }
        };

        return Ok(());
    }

    // Private functions used for testing
    fn get_page_count(&self) -> usize {
        self.pages.len()
    }

    fn set_prompts(&mut self, prompts: Prompts) {
        self.prompts = prompts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::test_utils::MockDB,
        model::{Epic, ItemDetail, ItemId, ItemStatus, Story},
    };

    #[test]
    fn should_start_on_home_page() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });
        let nav = Navigator::new(db);

        assert_eq!(nav.get_page_count(), 1);

        let current_page = nav.get_current_page().unwrap();
        let home_page = current_page.as_any().downcast_ref::<HomePage>();

        assert_eq!(home_page.is_some(), true);
    }

    #[test]
    fn handle_action_should_navigate_pages() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });

        let mut nav = Navigator::new(db);

        nav.handle_action(Action::NavigateToEpicDetail { epic_id: 1 })
            .unwrap();
        assert_eq!(nav.get_page_count(), 2);

        let current_page = nav.get_current_page().unwrap();
        let epic_detail_page = current_page.as_any().downcast_ref::<EpicDetail>();
        assert_eq!(epic_detail_page.is_some(), true);

        nav.handle_action(Action::NavigateToStoryDetail {
            epic_id: 1,
            story_id: 2,
        })
        .unwrap();
        assert_eq!(nav.get_page_count(), 3);

        let current_page = nav.get_current_page().unwrap();
        let story_detail_page = current_page.as_any().downcast_ref::<StoryDetail>();
        assert_eq!(story_detail_page.is_some(), true);

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 2);

        let current_page = nav.get_current_page().unwrap();
        let epic_detail_page = current_page.as_any().downcast_ref::<EpicDetail>();
        assert_eq!(epic_detail_page.is_some(), true);

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 1);

        let current_page = nav.get_current_page().unwrap();
        let home_page = current_page.as_any().downcast_ref::<HomePage>();
        assert_eq!(home_page.is_some(), true);

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 0);

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 0);
    }

    #[test]
    fn handle_action_should_clear_pages_on_exit() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });

        let mut nav = Navigator::new(db);

        nav.handle_action(Action::NavigateToEpicDetail { epic_id: 1 })
            .unwrap();
        nav.handle_action(Action::NavigateToStoryDetail {
            epic_id: 1,
            story_id: 2,
        })
        .unwrap();
        nav.handle_action(Action::Exit).unwrap();

        assert_eq!(nav.get_page_count(), 0);
    }

    #[test]
    fn handle_action_should_handle_create_epic() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.create_epic = Box::new(|| {
            Epic::new(
                ItemDetail {
                    name: "name".to_owned(),
                    description: "description".to_owned(),
                    id: ItemId(0),
                    status: ItemStatus::Open,
                },
                Vec::new(),
            )
        });

        nav.set_prompts(prompts);

        nav.handle_action(Action::CreateEpic).unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.epics.len(), 1);

        let epic = db_state.epics.into_iter().next().unwrap().1;
        assert_eq!(epic.detail.name, "name".to_owned());
        assert_eq!(epic.detail.description, "description".to_owned());
    }

    #[test]
    fn handle_action_should_handle_update_epic() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db.create_epic("".to_owned(), "".to_owned()).unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.update_status = Box::new(|| Some(ItemStatus::InProgress));

        nav.set_prompts(prompts);

        nav.handle_action(Action::UpdateEpicStatus { epic_id: epic_id.0 })
            .unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(
            db_state.epics.get(&epic_id.0).unwrap().detail.status,
            ItemStatus::InProgress
        );
    }

    #[test]
    fn handle_action_should_handle_delete_epic() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db.create_epic("".to_owned(), "".to_owned()).unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.delete_epic = Box::new(|| true);

        nav.set_prompts(prompts);

        nav.handle_action(Action::DeleteEpic { epic_id: epic_id.0 })
            .unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.epics.len(), 0);
    }

    #[test]
    fn handle_action_should_handle_create_story() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db.create_epic("".to_owned(), "".to_owned()).unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.create_story = Box::new(|| {
            Story::new(ItemDetail {
                name: "name".to_owned(),
                description: "description".to_owned(),
                id: ItemId(0),
                status: ItemStatus::Open,
            })
        });

        nav.set_prompts(prompts);

        nav.handle_action(Action::CreateStory { epic_id: epic_id.0 })
            .unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.stories.len(), 1);

        let story = db_state.stories.into_iter().next().unwrap().1;
        assert_eq!(story.detail.name, "name".to_owned());
        assert_eq!(story.detail.description, "description".to_owned());
    }

    #[test]
    fn handle_action_should_handle_update_story() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db.create_epic("".to_owned(), "".to_owned()).unwrap();
        let story_id = db
            .create_story("".to_owned(), "".to_owned(), Some(epic_id))
            .unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.update_status = Box::new(|| Some(ItemStatus::InProgress));

        nav.set_prompts(prompts);

        nav.handle_action(Action::UpdateStoryStatus {
            story_id: story_id.0,
        })
        .unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(
            db_state.stories.get(&story_id.0).unwrap().detail.status,
            ItemStatus::InProgress
        );
    }

    #[test]
    fn handle_action_should_handle_delete_story() {
        let db = Rc::new(JiraDataBase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db.create_epic("".to_owned(), "".to_owned()).unwrap();
        let story_id = db
            .create_story("".to_owned(), "".to_owned(), Some(epic_id))
            .unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.delete_story = Box::new(|| true);

        nav.set_prompts(prompts);

        nav.handle_action(Action::DeleteStory {
            epic_id: epic_id.0,
            story_id: story_id.0,
        })
        .unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.stories.len(), 0);
    }
}
