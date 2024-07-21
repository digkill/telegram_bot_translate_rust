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


    // We specify default parse mode to be `Html`, so that later we can use
    // `html::user_mention`
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
                    dptree::endpoint(repeat_message),
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

    // We get a "@username" mention via `mention()` method if the user has a
    // username, otherwise we create a textual mention with "Full Name" as the
    // text linking to the user
    let username = user.id;

    bot.send_message(chat_member.chat.id, format!("Welcome to {telegram_group_name} {username}!"))
        .await?;

    Ok(())
}

async fn left_chat_member(bot: Bot, chat_member: ChatMemberUpdated) -> ResponseResult<()> {
    let user = chat_member.old_chat_member.user;

    ////        user.mention().unwrap_or_else(|| html::user_mention(user.id, user.full_name().as_str()));
    let username = user.id;

    bot.send_message(chat_member.chat.id, format!("Goodbye {username}!")).await?;

    Ok(())
}

async fn repeat_message(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        let mut languages: [&str; 3] = ["th", "ru", "en"];

        match translate(text, &mut languages).await {
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

async fn translate(text: &str, target_languages: &mut [&str]) -> ResponseResult<String> {
    let client = reqwest::Client::new();

    let api_translate_url = env::var("API_TRANSLATE_URL").expect("API_TRANSLATE_URL not found");
    let api_translate_key = env::var("API_TRANSLATE_KEY").expect("API_TRANSLATE_KEY not found");
    let api_detect_url = env::var("API_DETECT_URL").expect("API_DETECT_URL not found");
    eprintln!("{text:#?}");

    let detectRequest = DetectRequest {
        q: String::from(text),
        api_key: String::from(api_translate_key.clone()),
    };


    let res = client.post(api_detect_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&detectRequest)
        // .form(&[("q", text), ("api_key", &api_translate_key_copy)])
        .send()
        .await?;
    let resp_json = res.json::<Vec<DetectResponse>>().await?;

    //  let resp_text = res.json::<DetectResponse>().await?;
    //  let json:DetectResponseList = serde_json::from_str(&resp_text).unwrap();
    let lang = &resp_json[0].language;

    eprintln!("{lang:#?}");

    /*
    println!("Status: {}", res.status());
    let body = res.text().await?;
    println!("Body: {}", body);*/
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
        .header("CONTENT_LENGTH", HeaderValue::from_static("0"))

        .send()
        .await?;

    // let response = res.text();

    //  eprintln!("{response:#?}");
    //        eprintln!("{u:#?}"); // Print the update to the console with inspect
    //  let resp_json = res.json().await?;

    // let resp_json = res.json::<HashMap<String, String>>().await?;
    //let resp_json = res.text().await?;

    let resp_json = res.json::<TranslateResult>().await?;

    //   let json_response = serde_json::from_str(&resp_json);

    let translated_word = resp_json.translatedText;

    //  let weather: WeatherResponse = response.json().await?;

    //  eprintln!("{translated_word:#?}");
    //  println!("{:#?}", resp_json);

    Ok(translated_word)
}