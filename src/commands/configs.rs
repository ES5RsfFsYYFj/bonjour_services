// Rust framework for Discord
use serenity::{
    client::{Context},

    model::{
        channel::Message,
    },

    framework::{
        standard::macros::command,
        standard::{CommandResult},
    },
};

//
// TODO: user could manage how they want to be welcomed
//
#[command]
async fn bonjour_config(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.reply(ctx, "Pong! Rust").await?;
    msg.channel_id.say(&ctx.http, "Pong! Rust from Ultrabot Bonjour services").await?;

    Ok(())
}