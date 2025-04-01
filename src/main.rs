use tokio::process::Command;
use serde_json::{Result, Value};
use std::collections::HashSet;
use std::result::Result as StdResult;
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use csv::ReaderBuilder;

fn parse_pipe_delimited_line(line: &str) -> Vec<String> {
    let sanitized = line.split('|').map(str::trim).collect::<Vec<_>>().join("|");
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .from_reader(sanitized.as_bytes());

    rdr.records()
        .next()
        .unwrap()
        .unwrap()
        .iter()
        .map(|s| s.trim().to_string())
        .collect()
}

#[tokio::main]
async fn main() -> StdResult<(),  Box<dyn Error>> { 


    let set: HashSet<&str> = ["brave-browser", "firefox", "chromium"]
        .iter()
        .cloned()
        .collect();
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines(); 
    let client = reqwest::Client::new();

    while let Ok(Some(line)) = lines.next_line().await {
        let my_line = parse_pipe_delimited_line(line.as_ref());
        if set.contains(&my_line[1].as_ref())  && my_line[4] == "tab" {
            let id = &my_line[3]; 

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
            let app = &my_line[1]; 
            let title = &my_line[2];
            println!("App: {}", app);
            let real_title = if !title.is_empty() {
                format!(" title=\"{}\"", title)
            } else {
                "".to_string()
            };

            println!("Title: {}", real_title);

            let arg_str = format!("[app_id=\"{}\"{}] focus", app, real_title );
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
