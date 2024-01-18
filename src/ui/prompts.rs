use crate::{
    io_utils::get_user_input,
    model::{Epic, ItemDetail, ItemId, ItemStatus, Story},
};

pub struct Prompts {
    pub create_epic: Box<dyn Fn() -> Epic>,
    pub create_story: Box<dyn Fn() -> Story>,
    pub delete_epic: Box<dyn Fn() -> bool>,
    pub delete_story: Box<dyn Fn() -> bool>,
    pub update_status: Box<dyn Fn() -> Option<ItemStatus>>,
}

impl Prompts {
    pub fn new() -> Self {
        Self {
            create_epic: Box::new(create_epic_prompt),
            create_story: Box::new(create_story_prompt),
            delete_epic: Box::new(delete_epic_prompt),
            delete_story: Box::new(delete_story_prompt),
            update_status: Box::new(update_status_prompt),
        }
    }
}

fn create_epic_prompt() -> Epic {
    println!("----------------------------");
    println!("Epic Name: ");
    let name = get_user_input();

    println!("Description: ");
    let description = get_user_input();

    return Epic::new(
        ItemDetail {
            name,
            description,
            id: ItemId(0),
            status: ItemStatus::Open,
        },
        Vec::new(),
    );
}

fn create_story_prompt() -> Story {
    println!("----------------------------");
    println!("Story Name: ");
    let name = get_user_input();

    println!("Description: ");
    let description = get_user_input();

    return Story::new(ItemDetail {
        description,
        id: ItemId(0),
        name,
        status: ItemStatus::Open,
    });
}

fn delete_epic_prompt() -> bool {
    println!("----------------------------");
    println!("Are you sure you want to delete this epic? All stories in this epic will also be deleted [Y/n]: ");

    let input = get_user_input();

    return input.eq("Y") || input.eq("y");
}

fn delete_story_prompt() -> bool {
    println!("----------------------------");
    println!("Are you sure you want to delete this story? [Y/n]: ");

    let input = get_user_input();

    return input.eq("Y") || input.eq("y");
}

fn update_status_prompt() -> Option<ItemStatus> {
    println!("----------------------------");
    println!("New Status (1 - OPEN, 2 - IN-PROGRESS, 3 - RESOLVED, 4 - CLOSED): ");

    let status = get_user_input();
    let status = status.trim().parse::<u8>();

    if let Ok(status) = status {
        return match status {
            1 => Some(ItemStatus::Open),
            2 => Some(ItemStatus::InProgress),
            3 => Some(ItemStatus::Resolved),
            4 => Some(ItemStatus::Closed),
            _ => None,
        };
    }

    None
}
