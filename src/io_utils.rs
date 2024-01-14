use std::io;

pub fn get_user_input() -> String {
  let mut input = String::new();

  io::stdin().read_line(&mut input).unwrap();

  return input;
}
