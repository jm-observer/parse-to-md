use anyhow::Result;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<()> {
    let rx_rename = Regex::new("rename_all = \"([^\"]*)\"").unwrap();
    let rx_tag = Regex::new("tag = \"([^\"]*)\"").unwrap();
    let rx_content = Regex::new("content = \"([^\"]*)\"").unwrap();

    let caps =
        rx_rename.captures(r#"rename_all = "snake_case""#).unwrap();
    for cap in caps.iter() {
        println!("{:?}", cap);
    }
    println!("{:?}", caps.get(1));
    Ok(())
}
