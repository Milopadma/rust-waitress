use std::env;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

use rawr::prelude::*;

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
    let client_id = env::var("REDDIT_CLIENT_ID").expect("client_id");
    let client_secret = env::var("REDDIT_CLIENT_SECRET").expect("client_secret");
    let client = rawr::Client::new("rustbot-rs", client_id, client_secret);

    // get the subreddit and post
    let subreddit = client.subreddit("LifeProTips");
    let post = subreddit
        .top()
        .limit(1)
        .fetch()
        .await
        .expect("Error fetching top post");
    println!("Resp: {:?}", post);

    msg.reply(
        ctx,
        format!(
            "Top post on r/rust is \"{}\" with {} points: {}",
            post.title, post.score, post.url
        ),
    )
    .await?;

    Ok(())
}
