pub mod ping;
pub mod configs;

use crate::commands::ping::PING_COMMAND;
use crate::commands::configs::BONJOUR_CONFIG_COMMAND;

// Rust framework for Discord
use serenity::framework::standard::macros::group;


#[group]
#[commands(ping, bonjour_config)]
struct General;