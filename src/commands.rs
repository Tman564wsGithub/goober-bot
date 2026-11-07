//! # SAFETY
//! A unwrap is fine if used on a `Guild`/`GuildId` in a `guild_only` poise function,
//! otherwise comment why an unwrap is better than error handling

mod clip;
mod join;
mod leave;

pub use clip::clip;
pub use join::join;
pub use leave::leave;
