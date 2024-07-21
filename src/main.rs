use teloxide::{prelude::*, types::ParseMode};
use teloxide_core::adaptors::DefaultParseMode;
use dotenv::dotenv;
use std::env;

type Bot = DefaultParseMode<teloxide::Bot>;

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
            eprintln!("{u:#?}"); // Print the update to the console with inspect
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


    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, text).await?;

        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}