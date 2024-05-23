use crate::notification::Notification;
use spectre_notify::{connection::ChannelConnection, notifier::Notifier};

pub type ConsensusNotifier = Notifier<Notification, ChannelConnection<Notification>>;
