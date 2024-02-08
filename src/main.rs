use std::{env, collections::HashSet};

use serenity::framework::standard::{StandardFramework, Configuration};
use serenity::model::gateway::GatewayIntents;
use serenity::all::Http;
use serenity::Client;

mod internals;
mod discord;
mod ollama;
use internals::*;
use discord::*;
use ollama::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info,ocllama=trace,serenity=warn,tracing=warn");
    }
    pretty_env_logger::init();
    log::info!("Starting Discord bot");

    let http = Http::new(&TOKEN);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new().before(before)
        .group(&USER_GROUP).group(&ADMIN_GROUP);

    framework.configure(
        Configuration::new().with_whitespace(true).on_mention(Some(bot_id))
            .prefix("/").owners(owners)
    );

    let intents = GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let mut client = match Client::builder(&*TOKEN, intents)
        .event_handler(Handler)
        .framework(framework).await {
            Err(err) => panic!("Unable to build discord client: {err:?}"),
            Ok(client) => client,
        };

    {
        let mut data = client.data.write().await;
        data.insert::<Ocllama>(Ocllama::new(
            (*OLLAMA_URI).to_owned(), *OLLAMA_PORT)
        );
    }

    if let Err(err) = client.start().await {
        panic!("Unable to start discord bot client: {err:?}");
    }
}
