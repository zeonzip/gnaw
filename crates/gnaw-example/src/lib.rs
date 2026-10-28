use std::fs;
use gnaw_lib::{shell, task};

#[task]
fn greet(name: String) {
    println!("Hello, {}!", name);
}