use crate::{
  types::{CommandContext, VoiceError},
  voice::leave_voice,
};

/// The command for the bot to leave the VC
#[poise::command(guild_only, slash_command)]
pub async fn leave(ctx: CommandContext<'_>) -> color_eyre::Result<()> {
  ctx.defer_ephemeral().await?;

  #[allow(clippy::unwrap_used)]
  let guild_id = ctx.guild_id().unwrap();
  if let Some(guild_data) = ctx.data().guilds.get(&guild_id) {
    guild_data.known_ssrcs.clear();
    guild_data.known_userids.clear();
    guild_data.voice_buffers.clear();
  }

  match leave_voice(ctx.serenity_context(), guild_id).await {
    Ok(()) => {
      ctx.reply("Left VC").await?;
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
