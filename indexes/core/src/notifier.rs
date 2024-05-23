use crate::notification::Notification;
use spectre_notify::{connection::ChannelConnection, notifier::Notifier};

pub type IndexNotifier = Notifier<Notification, ChannelConnection<Notification>>;
