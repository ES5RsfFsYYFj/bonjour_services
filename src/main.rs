pub mod commands;
pub mod events;

// Commands registration
use crate::commands::GENERAL_GROUP;

// Handler structure
use crate::events::general_handler;

// Rust framework for Discord
use serenity::{
    prelude::*,

    client::Client,

    framework:: standard::StandardFramework,

};

// Sound manager for Discord
use songbird::SerenityInit;

// Standard
use std::fs;

// JSON data handler
use serde_json;
use serde::{Deserialize};


#[non_exhaustive]
#[derive(Deserialize)]
struct Secrets
{
    // Bot's token
    token: String,

    // Directory where audio messages are stored or will be stored
    voice_directory: String,
} 


#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
                                            .configure(|c| c.prefix("-"))
                                            .group(&GENERAL_GROUP);


    // Load secrets from secret file
    let secrets_string = fs::read_to_string("./utils/secret.json")
                                    .expect("Cannot load secrets");
    
    // Parse values
    let secrets: Secrets = serde_json::from_str(&secrets_string)
                                                .expect("Parsing secrets file failed");

                                                
    let intents: GatewayIntents = GatewayIntents::non_privileged()
                                    | GatewayIntents::MESSAGE_CONTENT
                                    | GatewayIntents::GUILD_VOICE_STATES;

    
    let mut client = Client::builder(&secrets.token, intents)
                                        .event_handler(general_handler::Handler {
                                            voice_directory: secrets.voice_directory
                                        })
                                        .framework(framework)
                                        .register_songbird()
                                        .await
                                        .expect("Error creating client");


    // start listening for events by starting a single shard
    if let Err(why) = client.start().await
    {
        println!("An error occurred while running the client: {:?}", why);
    }

}

