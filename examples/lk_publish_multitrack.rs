use dotenvy::dotenv;
use livekit::{Room, RoomEvent, RoomOptions};

use livekit_api::access_token;
use rust_livekit_streamer::{
    GstVideoStream, LKParticipant, LKParticipantError, VideoPublishOptions,
};
use std::{env, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), LKParticipantError> {
    dotenv().ok();
    // Initialize gstreamer
    gstreamer::init().unwrap();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let url = env::var("LIVEKIT_URL").expect("LIVEKIT_URL is not set");
    let api_key = env::var("LIVEKIT_API_KEY").expect("LIVEKIT_API_KEY is not set");
    let api_secret = env::var("LIVEKIT_API_SECRET").expect("LIVEKIT_API_SECRET is not set");

    let token = access_token::AccessToken::with_api_key(&api_key, &api_secret)
        .with_identity("rust-bot-multivideo")
        .with_name("Rust Bot MultiVideo")
        .with_grants(access_token::VideoGrants {
            room_join: true,
            room: "DemoRoom".to_string(),
            ..Default::default()
        })
        .to_jwt()
        .unwrap();

    let (room, mut room_rx) = Room::connect(&url, &token, RoomOptions::default())
        .await
        .unwrap();

    let new_room = Arc::new(room);
    log::info!(
        "Connected to room: {} - {}",
        new_room.name(),
        String::from(new_room.sid().await)
    );

    let mut stream1 = GstVideoStream::new(VideoPublishOptions {
        codec: "image/jpeg".to_string(),
        width: 1920,
        height: 1080,
        framerate: 30,
        device_id: "/dev/video0".to_string(),
    });

    let mut stream2 = GstVideoStream::new(VideoPublishOptions {
        codec: "video/x-h264".to_string(),
        width: 1920,
        height: 1080,
        framerate: 30,
        device_id: "/dev/video4".to_string(),
    });

    stream1.start().await.unwrap();
    log::info!("Starting stream 1");

    stream2.start().await.unwrap();
    log::info!("Started stream 2");

    let mut participant = LKParticipant::new(new_room.clone());
    log::info!("Publishing stream 1");
    participant.publish_video_stream(&mut stream1, None).await?;
    log::info!("Starting stream 2");
    participant.publish_video_stream(&mut stream2, None).await?;

    while let Some(msg) = room_rx.recv().await {
        match msg {
            RoomEvent::Disconnected { reason } => {
                log::info!("Disconnected from room: {:?}", reason);
                stream1.stop().await?;
                stream2.stop().await?;
                break;
            }
            _ => {
                log::info!("Received room event: {:?}", msg);
            }
        }
    }

    Ok(())
}
