use crate::{
  types::{CommandContext, CommandError, VoiceError},
  voice::leave_voice,
};

/// The command for the bot to leave the VC
#[poise::command(guild_only, slash_command)]
pub async fn leave(ctx: CommandContext<'_>) -> Result<(), CommandError> {
  ctx.defer_ephemeral().await?;

  let Some(guild_id) = ctx.guild_id() else {
    ctx.reply("Not in a guild").await?;
    return Ok(());
  };

  if let Some(guild_data) = ctx.data().guilds.get(&guild_id) {
    guild_data.known_ssrcs.clear();
    guild_data.known_userids.clear();
    guild_data.voice_buffers.clear();
  }

  ctx.data().guilds.remove(&guild_id);

  match leave_voice(ctx.serenity_context(), guild_id).await {
    Ok(()) => {
      ctx.reply("Left VC").await?;
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
