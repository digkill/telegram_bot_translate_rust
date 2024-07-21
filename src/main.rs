use std::collections::HashMap;
use teloxide::{prelude::*, types::ParseMode};
use teloxide_core::adaptors::DefaultParseMode;
use dotenv::dotenv;
use std::env;
use std::iter::Map;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};

extern crate serde;
extern crate serde_json;

use serde_json::{json};

type Bot = DefaultParseMode<teloxide::Bot>;

#[derive(Serialize, Deserialize, Debug)]
struct TranslateResult {
    translatedText: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DetectResponse {
    language: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DetectRequest {
    q: String,
    api_key: String,
}

#[tokio::main]
async fn main() -> ResponseResult<()> {
    dotenv().ok();
    env_logger::init();
    let token_bot = env::var("TELEGRAM_BOT_KEY").expect("TELEGRAM_BOT_KEY not found");

    let bot = teloxide::Bot::new(&token_bot).parse_mode(ParseMode::Html);

    // Create a handler for our bot, that will process updates from Telegram
    let handler = dptree::entry()
        .inspect(|u: Update| {
            //        eprintln!("{u:#?}"); // Print the update to the console with inspect
        })
        .branch(
            Update::filter_chat_member()
                .branch(
                    dptree::filter(|m: ChatMemberUpdated| {
                        m.old_chat_member.is_left() && m.new_chat_member.is_present()
                    })
                        .endpoint(new_chat_member),
                )
                .branch(
                    dptree::filter(|m: ChatMemberUpdated| {
                        m.old_chat_member.is_present() && m.new_chat_member.is_left()
                    })
                        .endpoint(left_chat_member),
                )
        )
        .branch(
            Update::filter_message()
                .branch(
                    dptree::endpoint(translate_message),
                )
        );

    // Create a dispatcher for our bot
    Dispatcher::builder(bot, handler).enable_ctrlc_handler().build().dispatch().await;

    Ok(())
}

/// Welcome Endpoint
async fn new_chat_member(bot: Bot, chat_member: ChatMemberUpdated) -> ResponseResult<()> {
    let user = chat_member.old_chat_member.user.clone();

    let telegram_group_name = chat_member.chat.title().unwrap_or("");
    let username = user.id;

    bot.send_message(chat_member.chat.id, format!("Welcome to {telegram_group_name} {username}!"))
        .await?;

    Ok(())
}

async fn left_chat_member(bot: Bot, chat_member: ChatMemberUpdated) -> ResponseResult<()> {
    let user = chat_member.old_chat_member.user;
    let username = user.id;

    bot.send_message(chat_member.chat.id, format!("Goodbye {username}!")).await?;

    Ok(())
}

async fn translate_message(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {

        match translate(text).await {
            Ok(translated_word) => {
                bot.send_message(msg.chat.id, translated_word).await?;
            }
            Err(e) => {
                bot.send_message(msg.chat.id, format!("Translation error: {}", e)).await?;
            }
        }
    } else {
        bot.send_message(msg.chat.id, "Send me plain text.").await?;
    }

    Ok(())
}

async fn translate(text: &str) -> ResponseResult<String> {
    let client = reqwest::Client::new();

    let api_translate_url = env::var("API_TRANSLATE_URL").expect("API_TRANSLATE_URL not found");
    let api_translate_key = env::var("API_TRANSLATE_KEY").expect("API_TRANSLATE_KEY not found");
    let api_detect_url = env::var("API_DETECT_URL").expect("API_DETECT_URL not found");
    eprintln!("{text:#?}");

    let detect_request = DetectRequest {
        q: String::from(text),
        api_key: String::from(api_translate_key.clone()),
    };

    let res = client.post(api_detect_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&detect_request)
        .send()
        .await?;
    let resp_json = res.json::<Vec<DetectResponse>>().await?;

    let lang = &resp_json[0].language;

    eprintln!("{lang:#?}");
    let target_lang = if lang == "ru" { "th" } else { "ru" };

    let json_object = json!({
                "q": text,
                "source": "auto",
                "target": target_lang,
                "format": "text",
                "alternatives": 3,
                "api_key": api_translate_key
            });

    let json_string = serde_json::to_string(&json_object).unwrap();


    let res = client.post(api_translate_url)
        .body(json_string)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let resp_json = res.json::<TranslateResult>().await?;
    let translated_word = resp_json.translatedText;
    Ok(translated_word)
}