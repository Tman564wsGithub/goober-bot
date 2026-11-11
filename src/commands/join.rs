use serenity::all::ChannelId;

use crate::{
  types::{CommandContext, CommandError, VoiceError},
  voice::join_voice,
};

/// The command for the bot to join the VC
#[poise::command(guild_only, slash_command)]
pub async fn join(
  ctx: CommandContext<'_>,
  #[description = "Voice Channel"]
  #[channel_types("Voice")]
  voice_channel: ChannelId,
) -> Result<(), CommandError> {
  ctx.defer_ephemeral().await?;

  let Some(guild_id) = ctx.guild_id() else {
    ctx.reply("Not in a guild").await?;
    return Ok(());
  };

  let guild_data = ctx.data().guilds.entry(guild_id).or_default().clone();

  match join_voice(ctx.serenity_context(), guild_id, voice_channel, guild_data).await {
    Ok(()) => {
      ctx.reply("Joined VC").await?;
    }
    Err(VoiceError::VoiceModuleInitializing) => {
      ctx.reply("Voice module initializing..").await?;
    }
    Err(VoiceError::AlreadyInVC) => {
      ctx.reply("Already in VC").await?;
    }
    Err(VoiceError::NotInVC) => {
      ctx.reply("Not in VC").await?;
    }
    Err(VoiceError::SongbirdError(err)) => return Err(CommandError::SongbirdError(err)),
  }

  Ok(())
}
