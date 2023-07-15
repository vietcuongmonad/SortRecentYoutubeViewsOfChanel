#![allow(dead_code)]
#![allow(non_snake_case)]

use serde_json::Value;
use std::env;
use dotenv::dotenv;
use num_format::{Locale, ToFormattedString};
use iso8601_duration::Duration as IsoDuration;
use std::time::Duration;

async fn print_pretty_json(data: &Value) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&data)?);
    Ok(())
}

async fn response_data(url: String) -> Result<Value, Box<dyn std::error::Error>> {
    let response = reqwest::get(&url).await?.text().await?;
     
     Ok(serde_json::from_str(&response)?)
}

fn is_greater_than_one_minute(duration_str: &str) -> bool {
    let duration = duration_str.parse::<IsoDuration>().unwrap();
    let duration = Duration::from_secs(duration.to_std().unwrap().as_secs());
    duration > Duration::from_secs(60)
}

async fn get_channel_id(channel_name: String, api_key: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://www.googleapis.com/youtube/v3/search?part=snippet&type=channel&q={}&key={}", channel_name, api_key);

    let channel_data = response_data(url).await.unwrap();

    // print_pretty_json(&channel_data).await?;
    let channel_id = channel_data["items"][0]["snippet"]["channelId"].as_str().unwrap().to_string();

    Ok(channel_id)
}

async fn get_channel_videos(
    channel_id: String, 
    api_key: String, 
    max_recent_vid: u64, 
    no_short: Option<bool>
) 
-> Result<Vec<(String, u64, String, String)>, Box<dyn std::error::Error>> {
    let url = format!("https://www.googleapis.com/youtube/v3/channels?part=contentDetails&id={}&key={}", channel_id, api_key);

    // let url = format!("https://www.googleapis.com/youtube/v3/search?key={}&channelId={}&part=snippet,id&order=date&maxResults={}", api_key, channel_id, max_recent_vid);
    let channel_content = response_data(url).await.unwrap();

    let uploads_playlist_id = channel_content["items"][0]["contentDetails"]["relatedPlaylists"]["uploads"].as_str().unwrap();

    let url = format!("https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&key={}&maxResults={}", uploads_playlist_id, api_key, max_recent_vid);

    let video_data = response_data(url).await.unwrap();

    // print_pretty_json(&video_data).await?;

    let no_short = no_short.unwrap();

    let mut videos = Vec::new();

    let items = video_data["items"].as_array().unwrap();
    for item in items {
        
        let video_id = item["snippet"]["resourceId"]["videoId"].as_str().unwrap().to_string();
        let (view_count, duration) = get_view_count_and_duration(video_id.clone(), api_key.clone()).await?;

        // // println!("duration = {}", duration);
        if no_short && !is_greater_than_one_minute(&duration) {
            continue;
        }

        let title = item["snippet"]["title"].as_str().unwrap().to_string();
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

async fn get_view_count_and_duration(video_id: String, developer_key: String) -> Result<(u64, String), Box<dyn std::error::Error>> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?id={}&part=statistics,contentDetails&key={}",
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

    let duration = video_data["items"][0]["contentDetails"]["duration"].as_str().unwrap().to_string();


    Ok((view_count, duration))
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
    let channel_username = "emetsound";    // debug this
    let max_recent_vid = 100;
    let max_display_vid = 10;
    let no_shorts = false;

    dotenv().ok();
    let developer_key = env::var("DEVELOPER_KEY").expect("DEVELOPER_KEY must be set");

    match get_channel_id(channel_username.to_string().clone(), developer_key.clone()).await {
        Ok(channel_id) => {
            match get_channel_videos(channel_id, developer_key, max_recent_vid, Some(no_shorts)).await {
                Ok(videos) => display_vid(videos, max_display_vid),
                Err(e) => eprintln!("Error getting videos: \n{}", e)
            } 
        }    
        Err(e) => eprintln!("Error getting channel ID:\n{}", e)
    }

    Ok(())
}
