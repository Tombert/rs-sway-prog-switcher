use tokio::process::Command;
use serde_json::{Result, Value};
use std::collections::HashSet;
use std::result::Result as StdResult;
use std::{collections::HashMap, error::Error, future::Future, pin::Pin};
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

async fn tab_handler(my_line : Vec<String>) -> StdResult<(), Box<dyn Error + Send + Sync>>{
    let id = &my_line[3]; 
    let client = reqwest::Client::new();

    let s = format!("http://localhost:9222/json/activate/{}", id);
    println!("url {}",s);
    let _resp = client
        .post(s)
        .send()
        .await?;

    let _ = Command::new("swaymsg")
        .arg("[app_id=\"brave-browser\"] focus")
        .output()
        .await?;
    Ok(())
}

async fn default_handler(my_line: Vec<String>) -> StdResult<(), Box<dyn Error + Send + Sync>>
{

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
    let _ = Command::new("swaymsg")
        .arg(arg_str)
        //.arg("focus'")
        .output()
        .await?;
    Ok(())
}

type HandlerFn = Box<
    dyn Fn(Vec<String>) -> Pin<Box<dyn Future<Output = StdResult<(), Box<dyn Error + Send + Sync>>> + Send>>
        + Send
        + Sync,
>;

fn make_handler<F, Fut>(f: F) -> HandlerFn

where
    F: Fn(Vec<String>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = StdResult<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
{
    Box::new(move |args| Box::pin(f(args)))
}


async fn tmux_handler(my_line : Vec<String>) -> StdResult<(), Box<dyn Error + Send + Sync>>
{
            let id = &my_line[3];
            let tty = &my_line[1];
            let workspace = &my_line[0];

            let full_cmd = format!("tmux select-window -t {} \\; select-pane -t {}", workspace, id);
            let _ = Command::new("sh")
                .arg("-c")
                .arg(&full_cmd)
                .output()
                .await?;


            let _ = Command::new("swaymsg")
                .arg(format!("[app_id=\"{}\"] focus", tty))
                .output()
                .await?;
    Ok(())

}

#[tokio::main]
async fn main() -> StdResult<(),  Box<dyn Error>> { 
    let set: HashSet<&str> = ["brave-browser", "firefox", "chromium"]
        .iter()
        .cloned()
        .collect();
    let map: HashMap<String, HandlerFn> = vec![("tmux".to_string(), make_handler(tmux_handler)), ("tab".to_string(), make_handler(tab_handler))].into_iter().collect();
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines(); 
    let client = reqwest::Client::new();

    while let Ok(Some(line)) = lines.next_line().await {
        let my_line = parse_pipe_delimited_line(line.as_ref());
        let default = make_handler(default_handler);
        let handler = map.get(&my_line[4]).unwrap_or(&default);
        let _ = handler(my_line).await;
    }
    println!("howdy");
    Ok(())
}
