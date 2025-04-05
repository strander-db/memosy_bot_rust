use std::{env, error::Error, path::PathBuf, process::Command, time::Duration};

use teloxide::{
    prelude::*,
    types::{InputFile, MessageEntity, MessageEntityKind},
};
use yt_dlp::Youtube;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting memosy_bot_rust");
    let executables_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from("output");
    let fetcher = Youtube::with_new_binaries(executables_dir, output_dir).await?;
    tokio::spawn(async move {
        loop {
            match fetcher.update_downloader().await {
                Ok(_) => log::info!("Downloader updated"),
                Err(e) => log::error!("Failed to update downloader: {}", e),
            }
            tokio::time::sleep(Duration::from_secs(3600 * 24)).await;
        }
    });
    let bot = Bot::from_env();
    let admin_id = env::var("ADMIN_ID").expect("ADMIN_ID is not set");
    bot.send_message(admin_id, "Bot started").await?;
    teloxide::repl(bot, move |bot: Bot, msg: Message| async move {
        repl_handler(bot, msg).await
    })
    .await;
    Ok(())
}

async fn repl_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let sender = msg.from.clone();
    let sender_url = sender.as_ref().map(|sender| sender.id.url());
    let sender_name = sender.map(|sender| sender.first_name).unwrap_or_default();
    let urls = handle_message(&msg).await;
    for url in urls {
        let video = download_video(url.clone()).await;
        if let Ok(video) = video {
            let mut send_vid = bot.send_video(msg.chat.id, InputFile::file(video.clone()));
            let is_private = msg.chat.is_private();
            if !is_private {
                send_vid = send_vid.caption(format!(
                    "{}\n{}",
                    sender_name,
                    msg.text().unwrap_or_default()
                ));
                if let Some(ref sender_url) = sender_url {
                    send_vid = send_vid.caption_entities(vec![MessageEntity::text_link(
                        sender_url.clone(),
                        0,
                        sender_name.len(),
                    )]);
                }
            }
            send_vid.await?;
            if !is_private {
                bot.delete_message(msg.chat.id, msg.id).await?;
            }
            if let Err(e) = std::fs::remove_file(video) {
                log::error!("Failed to delete video file: {}", e);
            }
        }
    }
    Ok(())
}

async fn handle_message(msg: &Message) -> Vec<String> {
    if msg
        .text()
        .map(|text| text.contains("bot-ignore"))
        .unwrap_or(false)
    {
        return vec![];
    }
    if let Some(entities) = msg.parse_entities() {
        entities
            .iter()
            .filter(|entity| *entity.kind() == MessageEntityKind::Url)
            .map(|entity| String::from(entity.text()))
            .collect::<Vec<String>>()
    } else {
        vec![]
    }
}

async fn download_video(url: String) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let id = url
        .split("/")
        .last()
        .unwrap_or("video")
        .split("?")
        .next()
        .unwrap_or("video")
        .to_owned();
    let command = Command::new("./libs/yt-dlp")
        .arg("-f")
        .arg("best[ext=mp4]/best")
        .arg("-o")
        .arg(format!("{}.mp4", id))
        .arg(url)
        .arg("--impersonate")
        .arg("chrome")
        .status();
    if let Ok(status) = command {
        if status.success() {
            Ok(PathBuf::from(format!("{}.mp4", id)))
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to download video",
            )))
        }
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to download video",
        )))
    }
}
