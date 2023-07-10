from googleapiclient.discovery import build
from googleapiclient.errors import HttpError
from datetime import datetime, timedelta
from dotenv import load_dotenv

import isodate
import requests
import re
import json
import os

load_dotenv() # Same folder so no need to specify path

DEVELOPER_KEY = os.getenv('DEVELOPER_KEY')
YT_API_SERVICE_NAME = 'youtube'
YT_API_VERSION = 'v3'

def get_channel_id(channel_url):
    response = requests.get(channel_url)
    match = re.search(r'\/channel\/(.*?)\"', response.text)
    if match:
        return match.group(1)
    else:
        return None

def get_recent_vids(channel_id, months = 3, greaterLength = timedelta(minutes = 0)):
    youtube = build(YT_API_SERVICE_NAME, YT_API_VERSION, developerKey=DEVELOPER_KEY)

    # Get the channel's uploads playlist ID
    channels_response = youtube.channels().list(
        id = channel_id,
        part='contentDetails'
    ).execute()
    uploads_playlist_id = channels_response['items'][0]['contentDetails']['relatedPlaylists']['uploads']

    # Get videos in the uploads playlist
    videos = []
    next_page_token = ''

    flag_stop = False
    few_months_ago = datetime.now() - timedelta(days=months*30)
    
    while (next_page_token is not None):
        playlistitems_response = youtube.playlistItems().list(
            playlistId=uploads_playlist_id,
            part='snippet',
            maxResults=50,
            pageToken=next_page_token
        ).execute()

        # print(json.dumps(playlistitems_response, indent=4))

        for item in playlistitems_response['items']:
            published_at = datetime.strptime(item['snippet']['publishedAt'], '%Y-%m-%dT%H:%M:%SZ')
            if published_at < few_months_ago: 
                flag_stop = True
                break

            video_id = item['snippet']['resourceId']['videoId']

            video_response = youtube.videos().list(
                id=video_id,
                part='statistics,contentDetails'
            ).execute()

            duration = isodate.parse_duration(video_response['items'][0]['contentDetails']['duration'])
            if duration <= greaterLength:
                continue

            video_id = item['snippet']['resourceId']['videoId']            
            view_count = int(video_response['items'][0]['statistics']['viewCount'])
            video_url = f'https:///www.youtube.com/watch?v={video_id}'

            videos.append((item['snippet']['title'], view_count, video_url, duration))

        if flag_stop: break
        next_page_token = playlistitems_response.get('nextPageToken')
    
    return videos

def filterVideos(videos, display = 5):
    #print(json.dumps(videos, indent=4))
    sorted_videos = sorted(videos, key = lambda x: x[1], reverse=True)
    return sorted_videos[:display]

def output(videos):
    print(f'>> Top video uploaded in the last {months} months (by view):\n')
    for vid in videos:
        for feature in vid:
            if isinstance(feature, int): 
                print("    View count: ", format(feature, ','))
                continue

            print('   ', feature)

        print("\n")

def beautify(msg):
    if isinstance(msg, bytes):
        msg = json.loads(msg.decode('utf-8'))
    return json.dumps(msg, indent=4)

if __name__ == '__main__':
    try:
        channel_url = 'https://www.youtube.com/@alanbecker' # Debug: https://www.youtube.com/@Jw2pg
        months = 9
        display = 5 # How many vid
        greaterLength = timedelta(minutes=1)

        channel_id = get_channel_id(channel_url) #'UCObk_g1hQBy0RKKriVX_zOQ'
        vids = get_recent_vids(channel_id, months, greaterLength)
        vids = filterVideos(vids, display)
        output(vids)
    except HttpError as err:
        print(f'An HTTP error {err.resp.status} occurred:\n{beautify(err.content)}')
        