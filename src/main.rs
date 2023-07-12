#![allow(dead_code)]
#![allow(non_snake_case)]

use serde_json::Value;
use std::env;
use dotenv::dotenv;

use num_format::{Locale, ToFormattedString};

async fn print_pretty_json(data: &Value) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&data)?);
    Ok(())
}

async fn response_data(url: String) -> Result<Value, Box<dyn std::error::Error>> {
    let response = reqwest::get(&url).await?.text().await?;
     
     Ok(serde_json::from_str(&response)?)
}

async fn get_channel_id(channel_name: String, api_key: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://www.googleapis.com/youtube/v3/search?part=snippet&type=channel&q={}&key={}", channel_name, api_key);

    let channel_data = response_data(url).await.unwrap();
    let channel_id = channel_data["items"][0]["snippet"]["channelId"].as_str().unwrap().to_string();

    Ok(channel_id)
}

async fn get_channel_videos(channel_id: String, api_key: String, max_recent_vid: u64) 
-> Result<Vec<(String, u64, String, String)>, Box<dyn std::error::Error>> {
    let url = format!("https://www.googleapis.com/youtube/v3/channels?part=contentDetails&id={}&key={}", channel_id, api_key);

    // let url = format!("https://www.googleapis.com/youtube/v3/search?key={}&channelId={}&part=snippet,id&order=date&maxResults={}", api_key, channel_id, max_recent_vid);
    let channel_content = response_data(url).await.unwrap();

    let uploads_playlist_id = channel_content["items"][0]["contentDetails"]["relatedPlaylists"]["uploads"].as_str().unwrap();

    let url = format!("https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&key={}&maxResults={}", uploads_playlist_id, api_key, max_recent_vid);

    let video_data = response_data(url).await.unwrap();

    let mut videos = Vec::new();

    let items = video_data["items"].as_array().unwrap();
    for item in items {
        let title = item["snippet"]["title"].as_str().unwrap().to_string();
        let video_id = item["snippet"]["resourceId"]["videoId"].as_str().unwrap().to_string();
        let view_count = get_view_count(video_id.clone(), api_key.clone()).await?;
        let thumbnail = item["snippet"]["thumbnails"]["default"]["url"].as_str().unwrap().to_string();
        let video_url = format!("https://www.youtube.com/watch?v={}", video_id);

        videos.push((title, view_count, thumbnail, video_url));
    }

    videos.sort_by(
        |a, b|
        b.1.cmp(&a.1)
    );

    Ok(videos)
}

async fn get_view_count(video_id: String, developer_key: String) -> Result<u64, Box<dyn std::error::Error>> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?id={}&part=statistics&key={}",
        video_id,
        developer_key
    );
    let response = reqwest::get(&url).await?.text().await?;
    let video_data: Value = serde_json::from_str(&response)?;

    // print_pretty_json(&video_data).await?;

    let view_count = video_data["items"][0]["statistics"]["viewCount"]
        .as_str()
        .unwrap_or("0")
        .parse()
        .unwrap();

    Ok(view_count)
}

fn display_vid(videos: Vec<(String, u64, String, String)>, mut max_display_vid: u64) {
    for (title, view_count, thumbnail, video_url) in videos {
        println!("### {} \n### view count: {}\n", title, view_count.to_formatted_string(&Locale::en));
        println!("[![thumbnail]({})]({})\n", thumbnail, video_url);

        max_display_vid -= 1;
        if max_display_vid == 0 {
            break;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel_username = "ChocolateSundaes";
    let max_recent_vid = 100;
    let max_display_vid = 10;

    dotenv().ok();
    let developer_key = env::var("DEVELOPER_KEY").expect("DEVELOPER_KEY must be set");

    let channel_id = get_channel_id(channel_username.to_string().clone(), developer_key.clone()).await?;    
    let videos = get_channel_videos(channel_id, developer_key, max_recent_vid).await.unwrap();
    display_vid(videos, max_display_vid);

    Ok(())

    // let channel_url = format!("https://www.youtube.com/@{}", channel_username);
    // let months = 1;
    // let display = 5; // How many videos to display
    // let greater_length = Duration::from_secs(60); // 1 minute

    // let mut vids = get_recent_vids(channel_id, months, greater_length);
    // vids = filter_videos(vids, display);
    // output(vids);
}
