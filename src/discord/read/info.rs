use crate::discord::ids::{
    Id,
    marker::{ChannelMarker, MessageMarker},
};

/// One entry from `READY.read_state.entries[]`. The Discord wire field
/// `last_message_id` is renamed here because it actually carries the
/// last *ACKED* id, not the newest message in the channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadStateInfo {
    pub channel_id: Id<ChannelMarker>,
    pub last_acked_message_id: Option<Id<MessageMarker>>,
    pub mention_count: u32,
}
