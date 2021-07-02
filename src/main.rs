use std::error::Error;
use std::io::{self, Read};

mod eudcc;

fn main() -> Result<(), Box<dyn Error>> {
    let mut data = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut data)?;

    let certificate = eudcc::decode(data)?;
    println!("{:#?}", certificate);

    Ok(())
}
