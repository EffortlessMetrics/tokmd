use std::process::Command;

fn main() {
    println!("Checking if git is available...");
    let status = Command::new("git")
        .arg("--version")
        .status();

    match status {
        Ok(s) => println!("Git status: {:?}", s),
        Err(e) => println!("Git error: {:?}", e),
    }
}
