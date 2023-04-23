use std::env;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

// use rawr::prelude::*;
use rand::Rng;
use roux::Subreddit;

use reqwest::Error;
use serde::Deserialize;

use dotenv::dotenv;

#[derive(Debug, Deserialize)]
struct Response {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    content: Vec<Post>,
}

#[derive(Debug, Deserialize)]
struct Post {
    title: String,
    url: String,
    score: i32,
}

#[group]
#[commands(ping)]
#[commands(fetchtoppost)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Pong!");
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn fetchtoppost(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Fetching top post from r/rust");

    // random number from 0-5
    let rand_num: usize = rand::thread_rng().gen_range(0..5);

    // get the subreddit and post
    let subreddit = Subreddit::new("LifeProTips");
    let posts = subreddit.top(5, None).await.unwrap();
    // println!("Resp: {:?}", posts);

    msg.reply(
        ctx,
        format!(
            "Current Top post on r/LifeProTips is \"{}\" with {} points.",
            posts.data.children.get(rand_num).unwrap().data.title,
            posts.data.children.get(rand_num).unwrap().data.score,
        ),
    )
    .await?;

    // post an embedded URL to the text channel
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title(&posts.data.children.get(rand_num).unwrap().data.title);
                e.url(
                    &posts
                        .data
                        .children
                        .get(rand_num)
                        .unwrap()
                        .data
                        .url
                        .as_ref()
                        .unwrap(),
                );
                e
            });
            m
        })
        .await?;

    Ok(())
}
