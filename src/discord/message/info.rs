use crate::discord::ids::{
    Id,
    marker::{AttachmentMarker, ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker},
};

use crate::discord::commands::ReactionEmoji;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentionInfo {
    pub user_id: Id<UserMarker>,
    /// Per-server nickname carried by this message's mention payload. Kept
    /// separate from `display_name` so rendering can prefer a proven guild
    /// alias while still using cached member names when the payload only has a
    /// global display name or username.
    pub guild_nick: Option<String>,
    pub display_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentInfo {
    pub id: Id<AttachmentMarker>,
    pub filename: String,
    pub url: String,
    pub proxy_url: String,
    pub content_type: Option<String>,
    pub size: u64,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmbedFieldInfo {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmbedInfo {
    pub color: Option<u32>,
    pub provider_name: Option<String>,
    pub author_name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub timestamp: Option<String>,
    pub fields: Vec<EmbedFieldInfo>,
    pub footer_text: Option<String>,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub thumbnail_proxy_url: Option<String>,
    pub thumbnail_width: Option<u64>,
    pub thumbnail_height: Option<u64>,
    pub image_url: Option<String>,
    pub image_proxy_url: Option<String>,
    pub image_width: Option<u64>,
    pub image_height: Option<u64>,
    pub video_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InlinePreviewInfo<'a> {
    pub url: &'a str,
    pub proxy_url: Option<&'a str>,
    pub filename: &'a str,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub accent_color: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageKind {
    code: u8,
}

impl MessageKind {
    pub const fn new(code: u8) -> Self {
        Self { code }
    }

    pub const fn regular() -> Self {
        Self::new(0)
    }

    pub const fn code(self) -> u8 {
        self.code
    }

    pub const fn is_regular(self) -> bool {
        self.code == 0
    }

    pub const fn is_regular_or_reply(self) -> bool {
        // if it's a message or a reply to one
        self.code == 0 || self.code == 19
    }

    pub const fn known_label(self) -> Option<&'static str> {
        match self.code {
            0 => Some("Default"),
            1 => Some("Recipient add"),
            2 => Some("Recipient remove"),
            3 => Some("Call"),
            4 => Some("Channel name change"),
            5 => Some("Channel icon change"),
            6 => Some("Pinned message"),
            7 => Some("User join"),
            8 => Some("Guild boost"),
            9 => Some("Guild boost tier 1"),
            10 => Some("Guild boost tier 2"),
            11 => Some("Guild boost tier 3"),
            12 => Some("Channel follow add"),
            14 => Some("Guild discovery disqualified"),
            15 => Some("Guild discovery requalified"),
            16 => Some("Guild discovery initial warning"),
            17 => Some("Guild discovery final warning"),
            18 => Some("Thread created"),
            19 => Some("Reply"),
            20 => Some("Chat input command"),
            21 => Some("Thread starter message"),
            22 => Some("Guild invite reminder"),
            23 => Some("Context menu command"),
            24 => Some("Auto moderation action"),
            25 => Some("Role subscription purchase"),
            26 => Some("Premium upsell"),
            27 => Some("Stage start"),
            28 => Some("Stage end"),
            29 => Some("Stage speaker"),
            31 => Some("Stage topic"),
            32 => Some("Application premium subscription"),
            36 => Some("Incident alert mode enabled"),
            37 => Some("Incident alert mode disabled"),
            38 => Some("Incident raid report"),
            39 => Some("Incident false alarm report"),
            44 => Some("Purchase notification"),
            46 => Some("Poll result"),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self.known_label() {
            Some(label) => label,
            None => "Unknown message type",
        }
    }
}

impl Default for MessageKind {
    fn default() -> Self {
        Self::regular()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageSnapshotInfo {
    pub content: Option<String>,
    pub sticker_names: Vec<String>,
    pub mentions: Vec<MentionInfo>,
    pub attachments: Vec<AttachmentInfo>,
    pub embeds: Vec<EmbedInfo>,
    pub source_channel_id: Option<Id<ChannelMarker>>,
    pub timestamp: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplyInfo {
    pub author_id: Option<Id<UserMarker>>,
    pub author: String,
    pub content: Option<String>,
    pub sticker_names: Vec<String>,
    pub mentions: Vec<MentionInfo>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageInteractionInfo {
    pub user_id: Option<Id<UserMarker>>,
    pub user: String,
    pub command_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageReferenceInfo {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel_id: Option<Id<ChannelMarker>>,
    pub message_id: Option<Id<MessageMarker>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PollInfo {
    pub question: String,
    pub answers: Vec<PollAnswerInfo>,
    pub allow_multiselect: bool,
    pub results_finalized: Option<bool>,
    pub total_votes: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PollAnswerInfo {
    pub answer_id: u8,
    pub text: String,
    pub vote_count: Option<u64>,
    pub me_voted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionInfo {
    pub emoji: ReactionEmoji,
    pub count: u64,
    pub me: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionUserInfo {
    pub user_id: Id<UserMarker>,
    pub display_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionUsersInfo {
    pub emoji: ReactionEmoji,
    pub users: Vec<ReactionUserInfo>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageInfo {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel_id: Id<ChannelMarker>,
    pub message_id: Id<MessageMarker>,
    pub author_id: Id<UserMarker>,
    pub author: String,
    pub author_avatar_url: Option<String>,
    pub author_is_bot: bool,
    pub author_role_ids: Vec<Id<RoleMarker>>,
    pub message_kind: MessageKind,
    pub interaction: Option<MessageInteractionInfo>,
    pub reference: Option<MessageReferenceInfo>,
    pub reply: Option<ReplyInfo>,
    pub poll: Option<PollInfo>,
    pub pinned: bool,
    pub reactions: Vec<ReactionInfo>,
    pub content: Option<String>,
    pub sticker_names: Vec<String>,
    pub mentions: Vec<MentionInfo>,
    pub attachments: Vec<AttachmentInfo>,
    pub embeds: Vec<EmbedInfo>,
    pub forwarded_snapshots: Vec<MessageSnapshotInfo>,
    pub edited_timestamp: Option<String>,
}

impl Default for MessageInfo {
    fn default() -> Self {
        Self {
            guild_id: None,
            channel_id: Id::new(1),
            message_id: Id::new(1),
            author_id: Id::new(1),
            author: String::new(),
            author_avatar_url: None,
            author_is_bot: false,
            author_role_ids: Vec::new(),
            message_kind: MessageKind::default(),
            interaction: None,
            reference: None,
            reply: None,
            poll: None,
            pinned: false,
            reactions: Vec::new(),
            content: None,
            sticker_names: Vec::new(),
            mentions: Vec::new(),
            attachments: Vec::new(),
            embeds: Vec::new(),
            forwarded_snapshots: Vec::new(),
            edited_timestamp: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentUpdate {
    Unchanged,
    Replace(Vec<AttachmentInfo>),
}

impl AttachmentInfo {
    pub fn preferred_url(&self) -> Option<&str> {
        if self.url.is_empty() {
            (!self.proxy_url.is_empty()).then_some(self.proxy_url.as_str())
        } else {
            Some(self.url.as_str())
        }
    }

    pub fn is_image(&self) -> bool {
        if let Some(content_type) = self.content_type.as_deref() {
            return content_type.starts_with("image/");
        }

        filename_has_extension(
            &self.filename,
            &["avif", "gif", "jpeg", "jpg", "png", "webp"],
        )
    }

    pub fn is_video(&self) -> bool {
        if let Some(content_type) = self.content_type.as_deref() {
            return content_type.starts_with("video/");
        }

        filename_has_extension(&self.filename, &["m4v", "mov", "mp4", "webm"])
    }

    pub fn inline_preview_url(&self) -> Option<&str> {
        self.is_image().then(|| self.preferred_url()).flatten()
    }

    pub fn inline_preview_info(&self) -> Option<InlinePreviewInfo<'_>> {
        Some(InlinePreviewInfo {
            url: self.inline_preview_url()?,
            proxy_url: (!self.proxy_url.is_empty()).then_some(self.proxy_url.as_str()),
            filename: self.filename.as_str(),
            width: self.width,
            height: self.height,
            accent_color: None,
        })
    }
}

impl EmbedInfo {
    pub fn inline_preview_info(&self) -> Option<InlinePreviewInfo<'_>> {
        if let Some(url) = self.thumbnail_url.as_deref() {
            return Some(InlinePreviewInfo {
                url,
                proxy_url: self.thumbnail_proxy_url.as_deref(),
                filename: "embed-thumbnail",
                width: self.thumbnail_width,
                height: self.thumbnail_height,
                accent_color: Some(self.color.unwrap_or(0xff0000)),
            });
        }

        self.image_url.as_deref().map(|url| InlinePreviewInfo {
            url,
            proxy_url: self.image_proxy_url.as_deref(),
            filename: "embed-image",
            width: self.image_width,
            height: self.image_height,
            accent_color: Some(self.color.unwrap_or(0xff0000)),
        })
    }
}

fn filename_has_extension(filename: &str, extensions: &[&str]) -> bool {
    filename.rsplit_once('.').is_some_and(|(_, extension)| {
        extensions
            .iter()
            .any(|value| extension.eq_ignore_ascii_case(value))
    })
}
