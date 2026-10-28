use gnaw_lib::{shell, task};

#[task]
fn build() {
    shell! {
        "echo hi";
    }

    println!("Hello from Rust world!")
}

#[task]
fn other_task() {
    build(); // runs build first, aka dependencies!
    
    println!("Hello from other task!")
}