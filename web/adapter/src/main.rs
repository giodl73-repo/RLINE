use std::{env, fs::File, io::Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = env::args()
        .nth(1)
        .ok_or("Usage: rline-replay REQUEST.json | --examples")?;
    if arg == "--examples" {
        println!("{}", rline_web::examples());
        return Ok(());
    }
    let mut text = String::new();
    File::open(arg)?
        .take((rline_web::MAX_REQUEST + 1) as u64)
        .read_to_string(&mut text)?;
    println!("{}", rline_web::replay(&text));
    Ok(())
}
