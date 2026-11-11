use std::collections::HashMap;

use poise::CreateReply;
use serenity::all::{CreateAttachment, User, UserId};

use crate::{
  types::{CommandContext, CommandError},
  wave_utils::WaveWriter,
};

#[allow(clippy::unreadable_literal)]
const BOT_ID: u64 = 1549553408086638682;

/// The clip command
/// Can take an optional user to clip,
/// In which case it will output 1 wav file
#[poise::command(guild_only, slash_command)]
pub async fn clip(
  ctx: CommandContext<'_>,
  #[description = "Optional user to clip"] user: Option<User>,
) -> Result<(), CommandError> {
  ctx.defer().await?;

  let Some(guild_id) = ctx.guild_id() else {
    ctx.defer_ephemeral().await?;
    ctx.reply("Not in guild").await?;
    return Ok(());
  };

  let Some(guild_data) = ctx.data().guilds.get(&guild_id) else {
    ctx.defer_ephemeral().await?;
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
    ctx.defer_ephemeral().await?;
    reply = reply.content("I cannot clip myself");
    ctx.send(reply).await?;
    return Ok(());
  }

  reply = reply.content("Clipped!");
  // TODO: Create delete button

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
