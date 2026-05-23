use crate::discord::ids::{
    Id,
    marker::{RoleMarker, UserMarker},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberInfo {
    pub user_id: Id<UserMarker>,
    pub display_name: String,
    /// Discord login handle (`User.name`). Same role as in
    /// [`ChannelRecipientInfo::username`].
    pub username: Option<String>,
    pub is_bot: bool,
    pub avatar_url: Option<String>,
    pub role_ids: Vec<Id<RoleMarker>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleInfo {
    pub id: Id<RoleMarker>,
    pub name: String,
    pub color: Option<u32>,
    pub position: i64,
    pub hoist: bool,
    /// Discord permission bitfield carried by this role. Used by
    /// `DiscordState::can_view_channel` to compute base permissions and
    /// detect ADMINISTRATOR.
    pub permissions: u64,
}
