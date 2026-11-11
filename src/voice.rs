use std::sync::Arc;

use color_eyre::Result;
use serenity::{
  all::{ChannelId, Context, GuildId, UserId},
  async_trait,
};
use songbird::{
  CoreEvent, Event, EventContext,
  model::payload::{ClientDisconnect, Speaking},
};
use tracing::warn;

use crate::types::{GuildData, VoiceError};

const SILENCE_TICK: [i16; 48_000 / 50] = [0; 48_000 / 50];

#[derive(Clone)]
pub struct VoiceReceiver {
  pub inner: Arc<GuildData>,
}

#[async_trait]
impl songbird::EventHandler for VoiceReceiver {
  async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
    match ctx {
      EventContext::SpeakingStateUpdate(Speaking { ssrc, user_id, .. }) => {
        if let Some(user_id) = user_id {
          let user_id = UserId::from(user_id.0);
          self.inner.known_ssrcs.insert(*ssrc, user_id);
          self.inner.known_userids.insert(user_id, *ssrc);
        }
      }
      EventContext::VoiceTick(tick) => {
        for (ssrc, data) in &tick.speaking {
          let user_id = if let Some(id) = self.inner.known_ssrcs.get(ssrc) {
            *id.value()
          } else {
            continue;
          };

          if let Some(decoded_packet) = data.decoded_voice.as_ref() {
            self
              .inner
              .voice_buffers
              .entry(user_id)
              .or_default()
              .push_slice(decoded_packet);
          }
        }

        for ssrc in &tick.silent {
          let user_id = if let Some(id) = self.inner.known_ssrcs.get(ssrc) {
            *id.value()
          } else {
            continue;
          };

          self
            .inner
            .voice_buffers
            .entry(user_id)
            .or_default()
            .push_slice(&SILENCE_TICK);
        }
      }
      EventContext::ClientDisconnect(ClientDisconnect { user_id }) => {
        let user_id = UserId::from(user_id.0);
        if let Some(ssrc_ref) = self.inner.known_userids.get(&user_id) {
          let ssrc = *ssrc_ref;
          drop(ssrc_ref);
          self.inner.known_ssrcs.remove(&ssrc);
          self.inner.known_userids.remove(&user_id);
          self.inner.voice_buffers.remove(&user_id);
        }
      }
      _ => {
        warn!("Unimplemented function called in VoiceReceiver!");
      }
    }

    None
  }
}

/// Joins the `channel_id` if it exists.
///
/// # Errors
/// Will return `Err` if the function runs before songbird is ready, if the bot is already in a VC, or if `manager.join()` returns `Err`
pub async fn join_voice(
  ctx: &Context,
  guild_id: GuildId,
  channel_id: ChannelId,
  guild_data: Arc<GuildData>,
) -> Result<(), VoiceError> {
  let Some(manager) = songbird::get(ctx).await else {
    warn!("Tried to load songbird while still initializing");
    return Err(VoiceError::VoiceModuleInitializing);
  };

  if manager.get(guild_id).is_some() {
    return Err(VoiceError::AlreadyInVC);
  }

  {
    let handler_lock = manager.get_or_insert(guild_id);
    let mut handler = handler_lock.lock().await;

    let evt_receiver = VoiceReceiver { inner: guild_data };

    handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());
    handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver.clone());
    handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt_receiver);
  }

  if let Err(err) = manager.join(guild_id, channel_id).await {
    _ = manager.remove(guild_id).await;
    return Err(VoiceError::SongbirdError(err));
  }

  Ok(())
}

/// Leaves the `guild_id` if in a VC.
///
/// # Errors
/// Will return `Err` if the function runs before songbird is ready, if the bot isn't in a VC, or if `manager.remove()` returns `Err`
pub async fn leave_voice(ctx: &Context, guild_id: GuildId) -> Result<(), VoiceError> {
  let Some(manager) = songbird::get(ctx).await else {
    warn!("Tried to load songbird while still initializing");
    return Err(VoiceError::VoiceModuleInitializing);
  };

  if manager.get(guild_id).is_none() {
    return Err(VoiceError::NotInVC);
  }

  if let Err(err) = manager.remove(guild_id).await {
    return Err(VoiceError::SongbirdError(err));
  }

  Ok(())
}
