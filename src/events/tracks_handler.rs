// Rust framework for Discord
use serenity::{
    async_trait,

    model::{
        prelude::{GuildId, ChannelId}
    },
};

// Sound manager for Discord
use songbird::{
    Songbird,
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
};

// Standard
use std::sync::Arc;


pub struct EndTrackNotifier
{
    pub guild_id: GuildId,
    pub manager: Arc<Songbird>,
}

pub struct BeginTrackNotifier
{
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub manager: Arc<Songbird>,
}


#[async_trait]
impl VoiceEventHandler for BeginTrackNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event>
    {
        //
        // Moves to the channel where the user has to be welcomed 
        //
        match self.manager.join(self.guild_id, self.channel_id).await
        {
            (handler_lock, Ok(_)) => {
                println!("=== Sucessfully joint vocal chanel ===");
                            
                //
                // When moving between channels, the bot may consume the track before the user finishes the connection
                // Restart the track
                //
                match handler_lock.lock().await
                    .queue()
                    .current()
                    .expect("Cannot get current track")
                    .seek_time(std::time::Duration::new(0, 0))
                {
                    Err(why) => println!("Cannot restart track: {}", why),
                    _ => ()
                };
            },

            (_, Err(why)) => {
                println!("{}", why);
            }
        };
        None
    }
}

#[async_trait]
impl VoiceEventHandler for EndTrackNotifier {

    //
    // When a track has ended, check if there are others to play
    // if the job's done, leave the Discord server
    //
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event>
    {
        println!("Trigger end of track event");
        if let Some(handler_lock) = self.manager.get(self.guild_id)
        {
            let queue_is_empty = {
                    handler_lock.lock().await
                        .queue()
                        .is_empty()

            };
            if queue_is_empty
            {
                println!("Removing bot from voice channel");
                self.manager.remove(self.guild_id).await
                    .expect("Cannot remove bot from voice channel");
            }
        }
    
        None
    }
}
