use std::collections::HashMap;

use color_eyre::Result;
use poise::CreateReply;
use serenity::all::{ChannelId, CreateAttachment, User, UserId};

use crate::{
  types::{CommandContext, VoiceError},
  voice::{join_voice, leave_voice},
  wave_utils::WaveWriter,
};

#[allow(clippy::unreadable_literal)]
const BOT_ID: u64 = 1549553408086638682;

// TODO: Implement function to autocomplete all users IN the current call instead of just all users in guild (Or last few users in the chat, as discord does it right now!)

/// The command for the bot to join the VC
#[poise::command(guild_only, slash_command)]
pub async fn join(
  ctx: CommandContext<'_>,
  #[description = "Voice Channel"]
  #[channel_types("Voice")]
  voice_channel: ChannelId,
) -> Result<()> {
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

/// The command for the bot to leave the VC
#[poise::command(guild_only, slash_command)]
pub async fn leave(ctx: CommandContext<'_>) -> Result<()> {
  ctx.defer_ephemeral().await?;

  #[allow(clippy::unwrap_used)]
  let guild_id = ctx.guild_id().unwrap();

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

/// The clip command
/// Can take an optional user to clip,
/// In which case it will output 1 wav file
#[poise::command(guild_only, slash_command)]
pub async fn clip(
  ctx: CommandContext<'_>,
  #[description = "Optional user to clip"] user: Option<User>,
) -> Result<()> {
  ctx.defer().await?;

  #[allow(clippy::unwrap_used)]
  let guild_id = ctx.guild_id().unwrap();
  let Some(guild_data) = ctx.data().guilds.get(&guild_id) else {
    ctx.reply("Not in a VC").await?;
    return Ok(());
  };

  let voice_buffers: HashMap<UserId, Vec<i16>> = guild_data
    .voice_buffers
    .iter()
    .map(|x| (*x.key(), x.value().as_vec()))
    .collect();
  let mut reply = CreateReply::new();

  if let Some(user_id) = user.clone()
    && user_id.id == UserId::new(BOT_ID)
  {
    reply = reply.content("I cannot clip myself, silly!");
    ctx.send(reply).await?;
    return Ok(());
  }

  reply = reply.content("Clipped!");
  // TODO: Create delete button (`CreateButton`) with random id

  for (user_id, voice) in &voice_buffers {
    if let Some(user) = user.clone()
      && user.id != *user_id
    {
      continue;
    }

    let username = user_id.to_user(ctx).await?.name;
    let file_bytes: Vec<u8> = WaveWriter::new_wave(voice);
    reply = reply.attachment(CreateAttachment::bytes(
      file_bytes,
      format!("{username}-clip.wav"),
    ));
  }

  ctx.send(reply).await?;

  Ok(())
}
