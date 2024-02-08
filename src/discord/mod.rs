use async_trait::async_trait;

use serenity::framework::standard::macros::{group, check, hook};
use serenity::framework::standard::{CommandOptions, Reason, Args};
use serenity::model::{channel::Message, gateway::Ready};
pub use serenity::prelude::{TypeMapKey, Context};
pub use serenity::model::id::UserId;
use serenity::all::EventHandler;

use crate::internals::OWNER_ID;

mod admin;
mod user;
use admin::*;
use user::*;

#[group]
#[commands(setmodel, query, show, list)]
struct User;

#[group]
#[owners_only]
#[commands(deletemodel, addmodel)]
struct Admin;

#[check]
#[name = "Owner"]
async fn owner_check(_ctx: &Context, msg: &Message, _args: &mut Args, _opts: &CommandOptions) -> Result<(), Reason> {
    if msg.author.id != *OWNER_ID {
        Err(Reason::User("You ain't the admin of this bot".to_owned()))
    } else {
        Ok(())
    }
}

#[hook]
pub async fn before(_ctx: &Context, msg: &Message, cmd: &str) -> bool {
    log::trace!("Got command {cmd:?} by user {:?}", msg.author.name);
    true
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("Discord bot {} is connected!", ready.user.name);
    }
}

#[inline]
fn strip_cmd(msg: &str) -> String {
    let mut msg = msg.to_owned();
    msg.replace_range(..msg.find(' ').unwrap() + 1, "");
    msg
}
