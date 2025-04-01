use tokio::process::Command;
use serde_json::{Result, Value};
use std::collections::HashSet;
use std::result::Result as StdResult;
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> StdResult<(),  Box<dyn Error>> { 
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines(); 
    let client = reqwest::Client::new();

    while let Ok(Some(line)) = lines.next_line().await {
        let my_line : Vec<&str>= line.split('|').map(str::trim).collect();
        if my_line[1] == "brave-browser" && my_line.last().unwrap_or(&"").to_string() == "tab" {
            let id = my_line.get(my_line.len().wrapping_sub(2)).unwrap_or(&"");
            let id = id.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(id);

            let s = format!("http://localhost:9222/json/activate/{}", id);
            println!("url {}",s);
            let resp = client
                .post(s)
                .send()
                .await?;
            

            //swaymsg '[app_id="brave-browser"]  focus'
            let output = Command::new("swaymsg")
                .arg("[app_id=\"brave-browser\"] focus")
                //.arg("focus'")
                .output()
                .await?;
            println!("Status: {}", resp.status());
        } else {
            let app = my_line[1]; 
            let title = my_line[2];

            let arg_str = format!("[app_id={} title={}] focus", app, title);
            let output = Command::new("swaymsg")
                .arg(arg_str)
                //.arg("focus'")
                .output()
                .await?;

        }
        println!("Read: {}", my_line[0]);
        break; 
    }
    println!("howdy");
    Ok(())
}
