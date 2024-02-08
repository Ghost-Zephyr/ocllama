use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::builder::{CreateEmbed, CreateMessage, EditMessage};
use serenity::model::channel::Message;
pub use serenity::prelude::Context;

use super::strip_cmd;
use crate::Ocllama;

#[command]
#[aliases("q")]
async fn query(ctx: &Context, msg: &Message) -> CommandResult {
    let mut ansmsg = msg.channel_id.say(&ctx.http, "Thinking 🔃").await?;

    let data = ctx.data.read().await;
    let ocllama = data.get::<Ocllama>().unwrap();

    let query = strip_cmd(&msg.content);
    let answer = match ocllama.query(&msg.author.id, query.clone()).await {
        Ok(res) => res,
        Err(err) => {
            msg.channel_id.say(&ctx.http, err).await?;
            return Ok(());
        },
    };

    ansmsg.edit(&ctx.http, EditMessage::new().content(&answer.response)).await?;
    log::info!(
        "User {:?} queried {:?} {:?} and got answer {:?}",
        msg.author.name, answer.model, query, answer.response
    );
    Ok(())
}

#[command]
#[aliases("model")]
async fn setmodel(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let ocllama = data.get::<Ocllama>().unwrap();

    let mut model = strip_cmd(&msg.content);
    if model.find(':').is_none() {
        model += ":latest";
    }

    match ocllama.setmodel(&msg.author.id, model).await {
        Err(err) => msg.channel_id.say(&ctx.http, err).await?,
        Ok(model) => msg.channel_id.say(&ctx.http, format!(
            "Model changed to {model:?}")).await?,
    };
    Ok(())
}

#[command]
#[aliases("s")]
async fn show(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let ocllama = data.get::<Ocllama>().unwrap();
    let settings = ocllama.settings(&msg.author.id).await;

    let settings = CreateEmbed::new().title("Settings").field(
            "Model", settings.model, true);

    msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(settings)).await?;
    Ok(())
}

#[command]
#[aliases("l")]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let ocllama = data.get::<Ocllama>().unwrap();

    let mut fields = vec![];
    let mut inline_n = 0;
    let mut inline = std::iter::repeat_with(|| {
        inline_n += 1;
        if inline_n > 3 {
            inline_n = 0;
            false
        } else {
            true
        }
    });

    for model in match ocllama.list().await {
        Err(err) => {
            msg.channel_id.say(&ctx.http, err).await?;
            return Ok(());
        },
        Ok(models) => models,
    } {
        let info = match ocllama.info(model.name.clone()).await {
            Ok(info) => info,
            Err(err) => {
                fields.push((model.name.clone(), format!(
                    "Size: {}GB", model.size.div_ceil(1000u64.pow(3))
                ), inline.next().unwrap()));
                msg.channel_id.say(&ctx.http, format!(
                    "Error getting info for {:?}: {}",
                    model.name, err
                )).await?;
                continue
            },
        };
        let mut license = info.license.clone();
        license.replace_range(license.find('\n').unwrap().., "");

        fields.push((model.name, format!(
            "License: {}\nSize: {}GB\nTemplate: \"{}\"",
            license, model.size.div_ceil(1000u64.pow(3)), info.template
        ), inline.next().unwrap()));
    }
    let models = CreateEmbed::new().title("Available AI models").fields(fields);
    msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(models)).await?;
    Ok(())
}
