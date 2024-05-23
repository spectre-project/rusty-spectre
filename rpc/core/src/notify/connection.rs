use crate::Notification;

pub type ChannelConnection = spectre_notify::connection::ChannelConnection<Notification>;
pub use spectre_notify::connection::ChannelType;
