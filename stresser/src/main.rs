// Based on code from: http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust
//
// Change the header auth token and set the url to your test location.  Please do not test against
// the live, public system.  The urls.txt is just one url per line.
//

use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::env;
use std::process;
use futures::stream::StreamExt;
use rand::Rng;

const AUTH_HEADER_TOKEN: &str = "Blahblah^^12345678";
const USAGE_TEXT: &str = "stresser [hostname|ip] [url count]";
const MAX_CONCURRENT: usize = 50;

const REASONS: [&'static str; 3] = [
    "update",
    "live",
    "liveEnd",
];
const MEDIUMS: [&'static str; 14] = [
    "podcast",
    "podcastl",
    "music",
    "musicl",
    "video",
    "videol",
    "film",
    "filml",
    "audiobook",
    "audiobookl",
    "newsletter",
    "newsletterl",
    "blog",
    "blogl",
];


fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(
        reader.lines().filter_map(Result::ok).collect()
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let paths: Vec<String> = read_lines("urls.txt")?;

    let mut hostname = "";
    if let Some(arg_host) = &args.get(1) {
        hostname = arg_host;
    } else {
        eprintln!("{}", USAGE_TEXT);
        process::exit(1);
    }
    println!("Host: [{}].", hostname);

    //Urls to send
    let mut url_count = 5;
    if let Ok(arg_count) = args[2].parse() {
        url_count = arg_count;
    }
    println!("Sending: [{}] urls.", url_count);

    let fetches = futures::stream::iter(
    paths[..url_count].into_iter().map(|path| {
        async move {
            //Generate a random reason/medium
            let reason = REASONS.get(rand::thread_rng().gen_range(0..2)).unwrap();
            let medium = MEDIUMS.get(rand::thread_rng().gen_range(0..13)).unwrap();

            //Build the request url
            let pp_get_url = format!("http://{}/?url={}&medium={}&reason={}",
                hostname,
                path,
                medium,
                reason,
            );
            println!("Sending: [{}]...", pp_get_url);

            let client = reqwest::Client::new();
            match client.get(&pp_get_url)
              .header("Authorization", AUTH_HEADER_TOKEN)
              .header("User-Agent", "Stresser")
              .send()
              .await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok(text) => {
                            println!("RESPONSE: {} bytes from {}", text, pp_get_url);
                        }
                        Err(_) => println!("ERROR reading {}", pp_get_url),
                    }
                }
                Err(_) => println!("ERROR downloading {}", pp_get_url),
            }
        }
    })
    ).buffer_unordered(MAX_CONCURRENT).collect::<Vec<()>>();
    fetches.await;
    Ok(())
}
