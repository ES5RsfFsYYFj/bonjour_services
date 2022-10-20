use crate::events::tracks_handler;

// Rust framework for Discord
use serenity::{
    async_trait,

    client::{EventHandler, Context},

    model::{
        gateway::Ready,
        voice::VoiceState,
        id::UserId,
    },
};

// Sound manager for Discord
use songbird::{
    Event,
    TrackEvent,
    input::{
        restartable::Restartable,
        Input,
    }
};


// Standard
use std::{
    process::Command,
    path::Path,
    hash::{Hash, Hasher},
    collections::{hash_map::DefaultHasher},
};

// Date and Time
use chrono::{self, Timelike};


pub struct Handler
{
    pub voice_directory: String
}


#[async_trait]
impl EventHandler for Handler
{
    
    async fn ready(&self, _: Context, ready: Ready)
    {
        println!("{} is connected - Bonjour service", ready.user.name);
        
    }

    async fn voice_state_update(
        &self, 
        _ctx: Context,
        _old: Option<VoiceState>, 
        _new: VoiceState
    )
    {
        println!("+++ EVENT VOICE STATE UPDATE BEGIN +++");

        //
        // Handle only new connection in voice channels
        // If there is an old context, check if the channel id has changed from the new one (user's moving)
        // Otherwise don't process  
        //
        if let Some(old_context) = _old
        {
            let new_channel_id = _new.channel_id.expect("No channel id from new context");
            
            if old_context.channel_id.expect("No channel id from old context")
                    == new_channel_id
            {
                panic!("Handle only new connections");
            }
        };

        //
        // From UserId, gets User
        //
        let user_id: UserId = _new.user_id;
           
        //
        // Deny access to bots
        // TODO: Read values from list of bot in a config file
        //
        if user_id == 717678917418483764
            || user_id == 675345725512613899
        {
            panic!("{} action not handled : Do not process bot's events", user_id);
        }
        
        //
        // Generate user's welcoming audio file
        //
        let text_to_speech_hash: String;
        {
            let member = _new.member.expect("No member");

            //
            // Generate welcome's prologue 
            // Bonjour before 17h00
            // Bonsoir by 17h00
            //
            let time = chrono::Utc::now();
            let greeting = if time.hour() < 17 {"Bonjour"} else {"Bonsoir"};

            //
            // Generate whole welcome announce
            //
            let username = member.display_name();
            let text_to_speech = format!("{} {}", greeting, username);

            println!("=> Handle {} user", username);

            //
            // Calculate id from text
            //
            let mut hasher = DefaultHasher::new();
            text_to_speech.hash(&mut hasher);
            text_to_speech_hash = hasher.finish().to_string();

            //
            // Compare the text id with current files in the voices directory
            // If id is found don't query text to speech services
            // If text is new, query audio then cache it
            //
            if !(Path::new(&format!("{}/{}", self.voice_directory, text_to_speech_hash)).exists())
            {
                println!("Generate new audio");
                
                Command::new("/home/ultrae/.local/bin/gtts-cli")
                    .arg(text_to_speech)
                    .arg("--lang").arg("fr")
                    .arg("--output").arg(format!("{}/{}", self.voice_directory, text_to_speech_hash))
                    .output()
                    .expect("Cannot execute command");
            }
            
        }
        
        
        //
        // Welcomes users
        //
        if let Some(channel_id) = _new.channel_id
        {
            
            if let Some(guild_id) = _new.guild_id
            { 
                
                let manager = songbird::get(&_ctx).await
                                                            .expect("Cannot get manager")
                                                            .clone();
                
                let handler_lock = manager.get_or_insert(guild_id);
                
                //
                // We need to be connected in a voice channel in order to trigger events song
                //
                let is_first_connection = {

                    // We acquire the lock just to check is the bot is already connected in a voice channel
                    // We release it as soon as possible, as the lock is needed to join a channel, blocking 
                    // the access to the voice channel otherwise
                    let handler = handler_lock.lock().await;
                
                    handler.current_channel().is_none()
                };

                if is_first_connection 
                {
                    println!("Try to join first channel");
                    manager.join(guild_id, channel_id).await
                                .1
                                .expect("Cannot initiate first connection");
                }         
                
                //
                // Create an input which can be seekable
                // When moving between channels, the bot may consume the track before the user finishes the connection
                // Making the input seekable allow us to restart it at the right moment
                //
                let source = match Restartable::ffmpeg(
                                                    format!("{}/{}", self.voice_directory, text_to_speech_hash),
                                                    false).await
                {
                    Ok(input) => Input::from(input),
                    Err(why) => panic!("{}", why)
                };

                println!("Playing from event handler");
    
                //
                // Handle multplique tracks by stacking the welcome audio messages
                // When a track is dequeued and played, the bot jumps in the channel where the event was emitted
                // The bot exits when the queue's empty
                //
                let mut handler = handler_lock.lock().await;  

                let handle_tracker = handler.enqueue_source(source);

                // Triggered when a track is played
                handle_tracker.add_event(
                    Event::Track(TrackEvent::Play),
                    tracks_handler::BeginTrackNotifier {
                        guild_id: guild_id,
                        channel_id: channel_id,
                        manager: manager.clone(),
                    }
                ).expect("Cannot add Begin event for track");

                
                // Triggered when a track has ended
                handle_tracker.add_event(
                    Event::Track(TrackEvent::End),
                    tracks_handler::EndTrackNotifier {
                        guild_id: guild_id,
                        manager: manager.clone(),
                    }
                ).expect("Cannot add End event for track");
                
            };
        };
    }
}
