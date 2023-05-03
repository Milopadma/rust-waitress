use anyhow::anyhow;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};

use rand::Rng;

use roux::Subreddit;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
        // get the message content after "!subscribe"
        if msg.content.starts_with("!subscribe") {
            let messagesub = msg.content.trim_start_matches("!subscribe");
            let subreddit = Subreddit::new(messagesub.trim());

            if let Err(e) = msg
                .channel_id
                .say(&ctx.http, format!("Subscribed to{}", messagesub))
                .await
            {
                error!("Error sending message: {:?}", e);
            }

            // invoke async function loop that subcribes to that subreddit and sends posts to the discord channel every 24 hours
            subscribe(subreddit, msg, ctx).await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

async fn subscribe(subreddit: Subreddit, msg: Message, ctx: Context) {
    // loop that sends posts to the discord channel every 24 hours
    loop {
        // get the top post from the subreddit
        let posts = subreddit.hot(25, None).await.unwrap();

        // randomize get post
        let post = posts
            .data
            .children
            .get(rand::thread_rng().gen_range(0..5))
            .unwrap();
        // since the image url is an optional, we need to check if it exists
        // by matching if its a "self" or "default"
        let imageurl = match &post.data.thumbnail {
            url => match url.as_str() {
                "self" => None,
                "default" => None,
                _ => Some(url.as_str()),
            },
            _ => unreachable!("Thumbnail is not a string"),
        };

        println!("{:?}", &post.data.thumbnail);
        if let Err(e) = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&post.data.title)
                        .url(&post.data.url.as_ref().unwrap())
                        .description(&post.data.selftext)
                        .image(imageurl.unwrap_or("https://i.imgur.com/3QXVqyN.png"))
                })
            })
            .await
        {
            error!("Error sending message: {:?}", e);
        }

        // wait 24 hours
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
