#![recursion_limit = "256"]
use std::env;

use color_eyre::Result;
use dashmap::DashMap;
use poise::{BoxFuture, Framework, FrameworkError, FrameworkOptions};
use serenity::{
  all::{ActivityData, Ready},
  async_trait,
  prelude::*,
};
use songbird::{
  Config, SerenityInit,
  driver::{Channels, DecodeConfig, DecodeMode, SampleRate},
};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::types::{BotData, CommandError};

pub mod audio;
mod commands;
pub mod types;
pub mod voice;
pub mod wave_utils;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, ctx: Context, ready_data: Ready) {
    let version = env!("CARGO_PKG_VERSION");
    info!("{} is ready! (v{version})", ready_data.user.name);
    ctx.set_activity(Some(ActivityData::custom(format!("Running v{version}"))));
  }
}

fn on_command_error(error: FrameworkError<BotData, CommandError>) -> BoxFuture<()> {
  Box::pin(async move {
    match error {
      FrameworkError::Command { error, ctx, .. } => match error {
        CommandError::SongbirdError(err) => {
          error!("Error: Songbird error: {err}");
          _ = ctx
            .reply(format!("Internal error: Songbird error: {err}"))
            .await;
        }
        CommandError::SerenityError(err) => {
          error!("Error: Serenity error: {err}");
          _ = ctx
            .reply(format!("Internal error: Serenity error: {err}"))
            .await;
        }
      },
      other => {
        _ = poise::builtins::on_error(other).await;
      }
    }
  })
}

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt::fmt()
    .with_env_filter(EnvFilter::new(
      "warn,goober_bot=info,songbird::driver::tasks::udp_rx=off",
    ))
    .init();

  dotenvy::dotenv()?;

  let bot_token = env::var("BOT_TOKEN")?;
  let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT
    | GatewayIntents::GUILD_VOICE_STATES
    | GatewayIntents::GUILD_PRESENCES;

  let framework = Framework::builder()
    .options(FrameworkOptions {
      commands: vec![commands::join(), commands::clip(), commands::leave()],
      on_error: on_command_error,
      ..Default::default()
    })
    .setup(|ctx, _ready, framework| {
      Box::pin(async move {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
        Ok(BotData {
          guilds: DashMap::default(),
        })
      })
    })
    .build();

  let songbird_config = Config::default().decode_mode(DecodeMode::Decode(DecodeConfig::new(
    Channels::Mono,
    SampleRate::Hz48000,
  )));

  let mut bot = Client::builder(&bot_token, intents)
    .event_handler(Handler)
    .framework(framework)
    .register_songbird_from_config(songbird_config)
    .await?;

  if let Err(error) = bot.start().await {
    error!("Bot exited with error: {error}");
  }

  Ok(())
}
