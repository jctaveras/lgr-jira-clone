use std::io;

pub fn get_user_input() -> String {
    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();

    return input.trim().to_string();
}

pub fn wait_for_key_press() {
    io::stdin().read_line(&mut String::new()).unwrap();
}
