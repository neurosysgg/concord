use crate::discord::ChannelVisibilityStats;
use crate::tui::keybindings::{KeymapBindingSummary, SelectionAction};

use super::{
    ActiveGuildScope, DashboardState,
    popups::{KeymapPopupKind, KeymapPopupState},
};
use crate::logging;

fn clamp_keymap_popup_scroll(popup: &mut KeymapPopupState) {
    let max_scroll = popup.total_lines.saturating_sub(popup.view_height);
    popup.scroll = popup.scroll.min(max_scroll);
}

fn keymap_popup_half_page_distance(popup: &KeymapPopupState) -> usize {
    (popup.view_height / 2).max(1)
}

impl DashboardState {
    pub fn update_available_version(&self) -> Option<&str> {
        self.discord.update_available_version.as_deref()
    }

    pub fn gateway_error(&self) -> Option<&str> {
        self.runtime.gateway_error.as_deref()
    }

    pub fn is_debug_log_popup_open(&self) -> bool {
        self.popups.debug_log_popup_open
    }

    pub fn toggle_debug_log_popup(&mut self) {
        self.popups.debug_log_popup_open = !self.popups.debug_log_popup_open;
    }

    pub fn close_debug_log_popup(&mut self) {
        self.popups.debug_log_popup_open = false;
    }

    pub fn is_keymap_popup_open(&self) -> bool {
        self.popups.keymap_popup.is_some()
    }

    pub fn is_keymap_help_popup_open(&self) -> bool {
        self.popups
            .keymap_popup
            .as_ref()
            .is_some_and(|popup| popup.kind == KeymapPopupKind::Help)
    }

    pub fn open_keymap_help_popup(&mut self) {
        self.close_all_action_contexts();
        self.close_leader();
        self.popups.keymap_popup = Some(KeymapPopupState::new(KeymapPopupKind::Help));
    }

    pub fn close_keymap_popup(&mut self) {
        self.popups.keymap_popup = None;
    }

    pub fn keymap_popup_scroll(&self) -> usize {
        self.popups
            .keymap_popup
            .as_ref()
            .map(|popup| popup.scroll)
            .unwrap_or_default()
    }

    pub fn scroll_keymap_popup(&mut self, action: SelectionAction) {
        let Some(popup) = self.popups.keymap_popup.as_mut() else {
            return;
        };
        match action {
            SelectionAction::Next => popup.scroll = popup.scroll.saturating_add(1),
            SelectionAction::Previous => popup.scroll = popup.scroll.saturating_sub(1),
        }
        clamp_keymap_popup_scroll(popup);
    }

    pub fn page_keymap_popup_down(&mut self) {
        let Some(popup) = self.popups.keymap_popup.as_mut() else {
            return;
        };
        popup.scroll = popup
            .scroll
            .saturating_add(keymap_popup_half_page_distance(popup));
        clamp_keymap_popup_scroll(popup);
    }

    pub fn page_keymap_popup_up(&mut self) {
        let Some(popup) = self.popups.keymap_popup.as_mut() else {
            return;
        };
        popup.scroll = popup
            .scroll
            .saturating_sub(keymap_popup_half_page_distance(popup));
        clamp_keymap_popup_scroll(popup);
    }

    pub fn set_keymap_popup_view_height(&mut self, height: usize) {
        if let Some(popup) = self.popups.keymap_popup.as_mut() {
            popup.view_height = height;
            clamp_keymap_popup_scroll(popup);
        }
    }

    pub fn set_keymap_popup_total_lines(&mut self, total_lines: usize) {
        if let Some(popup) = self.popups.keymap_popup.as_mut() {
            popup.total_lines = total_lines;
            clamp_keymap_popup_scroll(popup);
        }
    }

    pub fn keymap_binding_summaries(&self) -> Vec<KeymapBindingSummary> {
        self.options.key_bindings.binding_summaries()
    }

    pub fn request_open_composer_in_editor(&mut self) {
        self.runtime.open_composer_in_editor_requested = true;
    }

    pub fn take_open_composer_in_editor_request(&mut self) -> bool {
        std::mem::take(&mut self.runtime.open_composer_in_editor_requested)
    }

    pub fn request_paste_clipboard(&mut self) {
        self.runtime.paste_clipboard_requested = true;
    }

    pub fn take_paste_clipboard_request(&mut self) -> bool {
        std::mem::take(&mut self.runtime.paste_clipboard_requested)
    }

    pub fn begin_clipboard_paste(&mut self) -> bool {
        if !self.is_composing() || self.runtime.clipboard_paste_pending {
            return false;
        }
        self.runtime.clipboard_paste_pending = true;
        true
    }

    pub fn finish_clipboard_paste(&mut self) {
        self.runtime.clipboard_paste_pending = false;
    }

    pub fn clipboard_paste_pending(&self) -> bool {
        self.runtime.clipboard_paste_pending
    }

    pub fn pending_composer_upload_line_count(&self) -> usize {
        self.composer.pending_composer_attachments.len()
            + usize::from(self.runtime.clipboard_paste_pending)
    }

    pub fn debug_log_lines(&self) -> Vec<String> {
        logging::error_entries()
            .into_iter()
            .map(|entry| entry.line())
            .collect()
    }

    /// Visible vs. permission-hidden channel counts for the active scope.
    /// Surfaced in the debug-log popup so the user can verify whether a
    /// missing channel is actually being filtered by `can_view_channel` or
    /// just isn't in the cache. DM scope always reports `(N, 0)`.
    pub fn debug_channel_visibility(&self) -> ChannelVisibilityStats {
        match self.navigation.active_guild {
            ActiveGuildScope::Unset => ChannelVisibilityStats::default(),
            ActiveGuildScope::DirectMessages => self.discord.cache.channel_visibility_stats(None),
            ActiveGuildScope::Guild(guild_id) => {
                self.discord.cache.channel_visibility_stats(Some(guild_id))
            }
        }
    }
}
