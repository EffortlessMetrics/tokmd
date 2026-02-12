use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let x: Option<i32> = None;
    x.ok_or("error")?;
    Ok(())
}
