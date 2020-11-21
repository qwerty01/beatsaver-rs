use std::error::Error;
use surf;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let uri = "https://httpbin.org/get";
    let string: String = surf::get(uri).recv_string().await?;
    println!("{}", string);

    Ok(())
}