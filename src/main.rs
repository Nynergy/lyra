// A basic usage example showing how to get the number of players
// currently connected to the LMS.

mod lms;

use lms::*;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = LmsClient::from("192.168.0.188:9000".to_string());

    let command = serde_json::json!(["-", ["serverstatus"]]);

    let res = client.query(command)
        .await?
        .json::<LmsResponse>()
        .await?;

    let count = res.get_u64("player count")
        .expect("Could not extract value");
    println!("Player Count: {}", count);

    res.dump();

    Ok(())
}
