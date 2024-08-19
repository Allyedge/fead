use std::error::Error;

use fetch::get_content;
use htmlentity::entity::{decode, ICodedDataTrait};

mod fetch;
mod reader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let resp = get_content("https://blog.rust-lang.org/feed.xml").await?;

    let data = reader::read_entries(resp.as_str())?;

    let first = data.first().unwrap();

    let title = &first.title.as_bytes().to_vec();
    let content = &first.content.as_bytes().to_vec();

    let decoded_title = decode(title);
    let decoded_content = decode(content);

    println!("{:?}", decoded_title.to_chars()?.iter().collect::<String>());
    println!(
        "{:?}",
        decoded_content.to_chars()?.iter().collect::<String>()
    );

    Ok(())
}
