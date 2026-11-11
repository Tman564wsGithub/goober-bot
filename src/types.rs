use std::sync::Arc;

use dashmap::DashMap;
use serenity::all::{GuildId, UserId};
use thiserror::Error;

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

pub type CommandContext<'a> = poise::Context<'a, BotData, CommandError>;

#[derive(Error, Debug)]
pub enum CommandError {
  #[error("serenity error: {0}")]
  SerenityError(#[from] Box<serenity::Error>),
  #[error("songbird error: {0}")]
  SongbirdError(#[from] songbird::error::JoinError),
}

// Yeah, this was just from AI, sorry no AI users (That sounds mean, but I do mean it, I really did not know how to convert a serenity::Error into a Box<serenity::Error> easily like this)
impl From<serenity::Error> for CommandError {
  fn from(e: serenity::Error) -> Self {
    Self::SerenityError(Box::new(e))
  }
}

#[derive(Error, Debug)]
pub enum VoiceError {
  #[error("not in vc")]
  NotInVC,
  #[error("already in vc")]
  AlreadyInVC,
  #[error("voice module initializing")]
  VoiceModuleInitializing,
  #[error("songbird error: {0}")]
  SongbirdError(#[from] songbird::error::JoinError),
}
