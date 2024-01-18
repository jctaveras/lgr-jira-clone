use std::rc::Rc;
use std::path::Path;

mod db;
mod io_utils;
mod model;
mod navigators;
mod ui;

use db::*;
use io_utils::*;
use navigators::*;

fn main() {
    let database = JiraDataBase::new(Path::new("database.json").to_path_buf());
    let mut navigator = Navigator::new(Rc::new(database));

    loop {
        match clearscreen::clear() {
            Ok(_) => {
                match navigator.get_current_page() {
                    None => break,
                    Some(page) => {
                        match page.draw_page() {
                            Ok(_) => {
                                let input = get_user_input();
                                let action = page.handle_input(input.trim());

                                match action {
                                    Ok(action) => {
                                        if let Some(action) = action {
                                            if let Err(error) = navigator.handle_action(action) {
                                                println!("Error handling user input: {error}");
                                                println!("Press any key to continue...");
                                                wait_for_key_press();
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        println!("Error while getting user input: {e}");
                                        println!("Press any key to continue...");
                                        wait_for_key_press();
                                    }
                                }
                            },
                            Err(e) => {
                                println!("Error while rendering page: {e}");
                                println!("Press any key to continue...");
                                wait_for_key_press();
                            }
                        }
                    }
                }
            }
            Err(_) => {
                println!("Something went wrong.");
                wait_for_key_press();
                break;
            }
        }
    }
}
