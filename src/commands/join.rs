use serenity::all::ChannelId;

use crate::{
  types::{CommandContext, VoiceError},
  voice::join_voice,
};

/// The command for the bot to join the VC
#[poise::command(guild_only, slash_command)]
pub async fn join(
  ctx: CommandContext<'_>,
  #[description = "Voice Channel"]
  #[channel_types("Voice")]
  voice_channel: ChannelId,
) -> color_eyre::Result<()> {
  ctx.defer_ephemeral().await?;

  #[allow(clippy::unwrap_used)]
  let guild_id = ctx.guild_id().unwrap();
  let guild_data = ctx.data().guilds.entry(guild_id).or_default().clone();

  match join_voice(ctx.serenity_context(), guild_id, voice_channel, guild_data).await {
    Ok(()) => {
      ctx.reply("Joined VC").await?;
    }
    Err(VoiceError::AlreadyInVC) => {
      ctx.reply("Already in a VC").await?;
    }
    Err(VoiceError::NotInVC) => {
      ctx.reply("Not in a VC").await?;
    }
    Err(VoiceError::SongbirdNotLoaded) => {
      ctx.reply("Songbird not loaded").await?;
    }
    Err(VoiceError::SongbirdError(err)) => {
      ctx.reply(format!("A error occurred: {err}")).await?;
    }
  }

  Ok(())
}
