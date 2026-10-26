use std::sync::Arc;

use dashmap::DashMap;
use songbird::model::id::{GuildId, UserId};

use crate::audio::AudioBuffer;

pub struct BotData {
  pub guilds: DashMap<GuildId, Arc<GuildData>>,
}

#[derive(Default, Clone)]
pub struct GuildData {
  pub known_ssrcs: DashMap<u32, UserId>,
  // Only because I need to reverse search when removing the user from known_ssrcs.
  pub known_userids: DashMap<UserId, u32>,
  pub voice_buffers: DashMap<UserId, AudioBuffer>,
}

pub type CommandContext<'a> = poise::Context<'a, BotData, color_eyre::Report>;
