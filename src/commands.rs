use color_eyre::Result;
use poise::CreateReply;
use serenity::all::{ChannelId, CreateAttachment, UserId};
use songbird::{CoreEvent, model::id::GuildId};

use crate::{types::CommandContext, voice::VoiceReceiver, wave_utils::WaveWriter};

/// The command for the bot to join the VC
#[poise::command(guild_only, slash_command)]
pub async fn join(
  ctx: CommandContext<'_>,
  #[description = "Voice Channel"] voice_channel: ChannelId,
) -> Result<()> {
  ctx.defer_ephemeral().await?;

  #[allow(clippy::unwrap_used)]
  let guild_id = ctx.guild_id().unwrap();
  let guild_data = ctx
    .data()
    .guilds
    .entry(GuildId(guild_id.get()))
    .or_default()
    .clone();

  let Some(manager) = songbird::get(ctx.serenity_context()).await else {
    ctx.reply("Songbird not initialised").await?;
    return Ok(());
  };

  {
    let handler_lock = manager.get_or_insert(guild_id);
    let mut handler = handler_lock.lock().await;

    let evt_receiver = VoiceReceiver {
      inner: guild_data.clone(),
    };

    handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());
    handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver.clone());
    handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt_receiver);
  }

  if let Ok(_handler) = manager.join(guild_id, voice_channel).await {
    ctx.reply("Joined VC").await?;
  } else {
    manager.leave(guild_id).await?;
    _ = ctx.reply("Could not join VC").await?;
  }

  Ok(())
}

/// The clip command
#[poise::command(guild_only, slash_command)]
pub async fn clip(
  ctx: CommandContext<'_>,
  #[description = "Optional user to clip"] user: Option<serenity::all::User>,
) -> Result<()> {
  ctx.defer().await?;

  #[allow(clippy::unwrap_used)]
  let guild_id = ctx.guild_id().unwrap();
  let Some(guild_data) = ctx.data().guilds.get(&GuildId(guild_id.get())) else {
    ctx.reply("Not in a VC").await?;
    return Ok(());
  };

  let voice_buffers: Vec<(songbird::model::id::UserId, Vec<i16>)> = guild_data
    .voice_buffers
    .iter()
    .map(|x| (*x.key(), x.value().as_vec()))
    .collect();
  let mut reply = CreateReply::new();
  reply = reply.content("Clipped!");

  for (user_id, voice) in &voice_buffers {
    let user_id = UserId::new(user_id.0);
    if let Some(user) = user.clone()
      && user.id != user_id
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

#[poise::command(guild_only, slash_command)]
pub async fn leave(ctx: CommandContext<'_>) -> Result<()> {
  ctx.defer_ephemeral().await?;

  #[allow(clippy::unwrap_used)]
  let guild_id = ctx.guild_id().unwrap();

  let Some(manager) = songbird::get(ctx.serenity_context()).await else {
    ctx.reply("Songbird not initialised").await?;
    return Ok(());
  };

  if let Ok(_leave) = manager.remove(guild_id).await {
    ctx.reply("Left VC").await?;
  } else {
    ctx.reply("Not in VC").await?;
  }

  Ok(())
}
