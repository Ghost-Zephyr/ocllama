use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
pub use serenity::prelude::Context;

use super::{OWNER_CHECK, strip_cmd};
use crate::Ocllama;

#[command]
#[checks(Owner)]
#[aliases("add")]
async fn addmodel(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let ocllama = data.get::<Ocllama>().unwrap();
    let model = strip_cmd(&msg.content);

    match ocllama.add_model(model).await {
        Err(err) => msg.channel_id.say(&ctx.http, err).await?,
        Ok(status) => {
            msg.channel_id.say(&ctx.http, status.message).await?
        },
    };
    Ok(())
}

#[command]
#[checks(Owner)]
#[aliases("rm")]
async fn deletemodel(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let ocllama = data.get::<Ocllama>().unwrap();
    let model = strip_cmd(&msg.content);

    match ocllama.delete_model(model).await {
        Err(err) => msg.channel_id.say(&ctx.http, err).await?,
        Ok(status) => {
            msg.channel_id.say(&ctx.http, status).await?
        },
    };
    Ok(())
}
