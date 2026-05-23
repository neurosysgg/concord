use crate::discord::ids::{
    Id,
    marker::{EmojiMarker, GuildMarker},
};

/// One entry from the user's `guild_folders` setting. A folder with `id ==
/// None` and a single member is an ungrouped guild. Discord stores those as
/// "folders" too just for ordering. Real folders carry an integer id, an
/// optional name, and an optional RGB color.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildFolder {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub color: Option<u32>,
    pub guild_ids: Vec<Id<GuildMarker>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomEmojiInfo {
    pub id: Id<EmojiMarker>,
    pub name: String,
    pub animated: bool,
    pub available: bool,
}
