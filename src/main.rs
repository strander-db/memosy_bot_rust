use std::{env, error::Error, path::PathBuf, process::Command, time::Duration};

use teloxide::{
    prelude::*,
    types::{InputFile, MessageEntity, MessageEntityKind},
};
use tokio::time::Instant;
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
    let timer = Instant::now();
    let sender = msg.from.clone();
    let sender_url = sender.as_ref().map(|sender| sender.id.url());
    let sender_name = sender.map(|sender| sender.first_name).unwrap_or_default();
    let urls = handle_message(&msg);
    log::info!("Handled message in: {:?}", timer.elapsed());
    for url in urls {
        let video = download_video(url.clone());
        log::info!("Downloaded video in: {:?}", timer.elapsed());
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
            log::info!("Sent video in: {:?}", timer.elapsed());
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

fn handle_message(msg: &Message) -> Vec<String> {
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

fn download_video(url: String) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let referrer = url.split('?').next().unwrap_or(&url).to_string();

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
        .arg("--user-agent")
        .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .arg("--add-header")
        .arg("Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .arg("--add-header")
        .arg("Accept-Language: en-US,en;q=0.9")
        .arg("--add-header")
        .arg("Accept-Encoding: gzip, deflate, br")
        .arg("--add-header")
        .arg("DNT: 1")
        .arg("--add-header")
        .arg("Sec-Fetch-Dest: document")
        .arg("--add-header")
        .arg("Sec-Fetch-Mode: navigate")
        .arg("--add-header")
        .arg("Sec-Fetch-Site: none")
        .arg("--add-header")
        .arg("Sec-Fetch-User: ?1")
        .arg("--add-header")
        .arg("Upgrade-Insecure-Requests: 1")
        .arg("--add-header")
        .arg("sec-ch-ua: \"Chromium\";v=\"122\", \"Google Chrome\";v=\"122\", \"Not(A:Brand\";v=\"24\"")
        .arg("--add-header")
        .arg("sec-ch-ua-mobile: ?0")
        .arg("--add-header")
        .arg("sec-ch-ua-platform: \"Windows\"")
        .arg("--referer")
        .arg(&referrer)
        .arg("--add-header")
        .arg(format!("Origin: {}", referrer.split('/').take(3).collect::<Vec<_>>().join("/")))
        .arg("--add-header")
        .arg("Sec-Fetch-Site: same-origin")
        .arg("--impersonate")
        .arg("chrome")
        .arg(url)
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
