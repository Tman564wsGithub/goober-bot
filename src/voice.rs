use std::sync::Arc;

use serenity::async_trait;
use songbird::{
  Event, EventContext,
  model::{
    id::UserId,
    payload::{ClientDisconnect, Speaking},
  },
};
use tracing::warn;

use crate::types::GuildData;

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
          self.inner.known_ssrcs.insert(*ssrc, *user_id);
          self.inner.known_userids.insert(*user_id, *ssrc);
        }
      }
      EventContext::VoiceTick(tick) => {
        for (ssrc, data) in &tick.speaking {
          let user_id = if let Some(id) = self.inner.known_ssrcs.get(ssrc) {
            id.0
          } else {
            continue;
          };

          if let Some(decoded_packet) = data.decoded_voice.as_ref() {
            self
              .inner
              .voice_buffers
              .entry(UserId(user_id))
              .or_default()
              .push_slice(decoded_packet);
          }
        }

        for ssrc in &tick.silent {
          let user_id = if let Some(id) = self.inner.known_ssrcs.get(ssrc) {
            id.0
          } else {
            continue;
          };

          self
            .inner
            .voice_buffers
            .entry(UserId(user_id))
            .or_default()
            .push_slice(&SILENCE_TICK);
        }
      }
      EventContext::ClientDisconnect(ClientDisconnect { user_id }) => {
        if let Some(ssrc_ref) = self.inner.known_userids.get(user_id) {
          let ssrc = *ssrc_ref;
          drop(ssrc_ref);
          self.inner.known_ssrcs.remove(&ssrc);
          self.inner.known_userids.remove(user_id);
          self.inner.voice_buffers.remove(user_id);
        }
      }
      _ => {
        warn!("Unimplemented function called in VoiceReceiver!");
      }
    }

    None
  }
}
