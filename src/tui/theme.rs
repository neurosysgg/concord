use std::sync::OnceLock;

use ratatui::style::Color;

#[derive(Clone, Copy, Debug)]
pub(super) struct Theme {
    pub(super) accent: Color,
    pub(super) dim: Color,
    pub(super) border: Color,
    pub(super) background: Color,
    pub(super) text: Color,
    pub(super) active: Color,
    pub(super) panel_title: Color,
    /// Discord's "you were mentioned" orange, `#FFA500`.
    pub(super) mention: Color,
    /// Explicit RGB instead of `Modifier::DIM` so CJK wide characters dim
    /// uniformly with ASCII (most terminals ignore SGR dim on wide glyphs).
    pub(super) read_dim: Color,
    /// Explicit RGB instead of relying on `Modifier::BOLD` alone, which most
    /// monospace fonts can't apply to CJK glyphs.
    pub(super) unread_bright: Color,
    pub(super) scrollbar_thumb: Color,
    pub(super) selected_forum_post_border: Color,
    pub(super) selected_message_border: Color,
    pub(super) success: Color,
    pub(super) warning: Color,
    pub(super) error: Color,
    pub(super) info: Color,
    /// Discord's brand blurple, `#5865F2`.
    pub(super) blurple: Color,
    pub(super) selection_bg: Color,
    /// Discord's unread-divider red, distinct from `error`.
    pub(super) unread_badge: Color,
    pub(super) self_reaction: Color,
    /// Discord's gold "you were mentioned" text highlight background.
    pub(super) mention_self_bg: Color,
    /// Discord's blue "someone else was mentioned" text highlight background.
    pub(super) mention_other_bg: Color,
    pub(super) mention_other_fg: Color,
    pub(super) presence_idle: Color,
    pub(super) mention_role_fallback: Color,
    pub(super) dm_icon: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Cyan,
            dim: Color::DarkGray,
            border: Color::DarkGray,
            background: Color::Rgb(24, 54, 65),
            text: Color::White,
            active: Color::Green,
            panel_title: Color::Reset,
            mention: Color::Rgb(255, 165, 0),
            read_dim: Color::Rgb(130, 130, 130),
            unread_bright: Color::Reset,
            scrollbar_thumb: Color::Rgb(170, 170, 170),
            selected_forum_post_border: Color::Green,
            selected_message_border: Color::Green,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            blurple: Color::Rgb(88, 101, 242),
            selection_bg: Color::Rgb(40, 45, 90),
            unread_badge: Color::Rgb(237, 66, 69),
            self_reaction: Color::Yellow,
            mention_self_bg: Color::Rgb(92, 76, 35),
            mention_other_bg: Color::Rgb(40, 50, 92),
            mention_other_fg: Color::Rgb(193, 206, 247),
            presence_idle: Color::Rgb(180, 140, 0),
            mention_role_fallback: Color::Magenta,
            dm_icon: Color::Magenta,
        }
    }
}

static THEME: OnceLock<Theme> = OnceLock::new();

#[allow(dead_code)]
pub(super) fn init(theme: Theme) {
    let _ = THEME.set(theme);
}

pub(super) fn current() -> Theme {
    *THEME.get_or_init(Theme::default)
}
