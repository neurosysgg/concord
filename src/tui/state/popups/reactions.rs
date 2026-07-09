use crate::discord::ids::{
    Id,
    marker::{ChannelMarker, GuildMarker, MessageMarker},
};
use crate::discord::{AppCommand, MessageState, ReactionEmoji};
use crate::tui::fuzzy::{FuzzyScore, fuzzy_text_score};
use crate::tui::keybindings::SelectionAction;

use super::super::emoji::{
    custom_emoji_can_be_used_directly, custom_emoji_reaction_item, is_quick_unicode_emoji,
    quick_unicode_emoji_reaction_items, remaining_unicode_emoji_reaction_items,
};
use super::super::{
    DashboardState, EmojiReactionItem, EmojiReactionPickerState, ReactionUsersPopupState,
};
use crate::tui::state::popups::{ActiveModalPopupKind, ModalPopup};

impl DashboardState {
    pub fn reaction_users_popup(&self) -> Option<&ReactionUsersPopupState> {
        self.popups.reaction_users_popup()
    }

    pub fn emoji_reaction_items(&self) -> Vec<EmojiReactionItem> {
        if let Some(picker) = self.popups.emoji_reaction_picker() {
            return picker.items.clone();
        }

        self.emoji_reaction_items_for_guild(self.picker_guild_id())
    }

    fn emoji_reaction_items_for_guild(
        &self,
        guild_id: Option<Id<GuildMarker>>,
    ) -> Vec<EmojiReactionItem> {
        let mut items = quick_unicode_emoji_reaction_items();

        if let Some(guild_id) = guild_id {
            items.extend(
                self.discord
                    .cache
                    .custom_emojis_for_guild(guild_id)
                    .iter()
                    .filter(|emoji| {
                        emoji.available
                            && custom_emoji_can_be_used_directly(
                                emoji,
                                false,
                                self.current_user_has_nitro(),
                            )
                    })
                    .map(custom_emoji_reaction_item),
            );
        }

        if self.current_user_has_nitro() {
            items.extend(
                self.discord
                    .cache
                    .all_custom_emojis()
                    .filter(|(emoji_guild_id, _)| {
                        guild_id.is_none_or(|guild_id| guild_id != **emoji_guild_id)
                    })
                    .flat_map(|(_, emojis)| emojis)
                    .filter(|emoji| {
                        emoji.available
                            && custom_emoji_can_be_used_directly(
                                emoji,
                                true,
                                self.current_user_has_nitro(),
                            )
                    })
                    .map(custom_emoji_reaction_item),
            );
        }

        items.extend(remaining_unicode_emoji_reaction_items());

        for item in items.iter_mut() {
            item.is_pinned = self.reactions.pinned_emojis.contains(&item.emoji);
        }

        // A pinned emoji only ever shows up once, at the very front, instead
        // of also sitting wherever it'd normally fall in the catalog.
        let pinned = self.pinned_emoji_reaction_items(&items);
        items.retain(|item| !item.is_pinned);

        let mut result = pinned;
        result.extend(items);
        result
    }

    fn pinned_emoji_reaction_items(&self, catalog: &[EmojiReactionItem]) -> Vec<EmojiReactionItem> {
        self.reactions
            .pinned_emojis
            .iter()
            .filter_map(|emoji| catalog.iter().find(|item| &item.emoji == emoji).cloned())
            .collect()
    }

    pub fn filtered_emoji_reaction_items(&self) -> Vec<EmojiReactionItem> {
        if let Some(picker) = self.popups.emoji_reaction_picker() {
            return picker.filtered_items.clone();
        }

        let items = self.emoji_reaction_items();
        let Some(filter) = self.emoji_reaction_filter() else {
            return items;
        };

        filter_emoji_reaction_items(items, filter)
    }

    pub fn filtered_emoji_reaction_items_slice(&self) -> Option<&[EmojiReactionItem]> {
        self.popups
            .emoji_reaction_picker()
            .map(|picker| picker.filtered_items.as_slice())
    }

    pub fn emoji_reaction_filter(&self) -> Option<&str> {
        self.popups
            .emoji_reaction_picker()
            .and_then(|picker| picker.filter.as_deref())
    }

    pub fn existing_emoji_reactions(&self) -> &[crate::discord::ReactionEmoji] {
        self.popups
            .emoji_reaction_picker()
            .map(|picker| picker.existing_reactions.as_slice())
            .unwrap_or(&[])
    }

    pub fn own_emoji_reactions(&self) -> &[crate::discord::ReactionEmoji] {
        self.popups
            .emoji_reaction_picker()
            .map(|picker| picker.own_reactions.as_slice())
            .unwrap_or(&[])
    }

    pub fn is_filtering_emoji_reactions(&self) -> bool {
        self.emoji_reaction_filter().is_some()
    }

    pub fn is_editing_emoji_reaction_filter(&self) -> bool {
        self.popups
            .emoji_reaction_picker()
            .is_some_and(|picker| picker.filter_editing)
    }

    pub fn close_emoji_reaction_picker(&mut self) {
        if self.is_active_modal_popup(ActiveModalPopupKind::EmojiReactionPicker) {
            self.popups.clear_modal();
        }
    }

    pub fn close_reaction_users_popup(&mut self) {
        if self.is_active_modal_popup(ActiveModalPopupKind::ReactionUsers) {
            self.popups.clear_modal();
        }
    }

    pub fn open_reaction_users_popup(
        &mut self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
        reactions: Vec<(ReactionEmoji, u64)>,
    ) {
        let popup = ReactionUsersPopupState::new(channel_id, message_id, reactions);
        self.popups.modal = Some(ModalPopup::ReactionUsers(popup));
    }

    pub fn navigate_reaction_users_popup(&mut self, action: SelectionAction) -> Option<AppCommand> {
        let popup = self.popups.reaction_users_popup_mut()?;
        if !popup.is_viewing_users() {
            popup.move_selection(action);
            return None;
        }
        match action {
            SelectionAction::Previous => {
                popup.user_scroll.scroll_up();
                None
            }
            SelectionAction::Next => {
                popup.user_scroll.scroll_down();
                let (emoji, after) = popup.take_load_more()?;
                Some(AppCommand::LoadReactionUsers {
                    channel_id: popup.channel_id,
                    message_id: popup.message_id,
                    emoji,
                    after: Some(after),
                })
            }
        }
    }

    /// Fetches the next page after a page or half-page scroll of the user list,
    /// when the reader is near the bottom and more users remain.
    pub fn reaction_users_popup_take_load_more(&mut self) -> Option<AppCommand> {
        let popup = self.popups.reaction_users_popup_mut()?;
        let (emoji, after) = popup.take_load_more()?;
        Some(AppCommand::LoadReactionUsers {
            channel_id: popup.channel_id,
            message_id: popup.message_id,
            emoji,
            after: Some(after),
        })
    }

    pub fn activate_reaction_users_popup(&mut self) -> Option<AppCommand> {
        let popup = self.popups.reaction_users_popup_mut()?;
        if popup.is_viewing_users() {
            return None;
        }
        let emoji = popup.open_selected()?;
        Some(AppCommand::LoadReactionUsers {
            channel_id: popup.channel_id,
            message_id: popup.message_id,
            emoji,
            after: None,
        })
    }

    pub fn reaction_users_popup_back(&mut self) -> bool {
        self.popups
            .reaction_users_popup_mut()
            .is_some_and(|popup| popup.back_to_list())
    }

    pub fn set_reaction_users_popup_view_height(&mut self, height: usize) {
        if let Some(popup) = self.popups.reaction_users_popup_mut() {
            if popup.is_viewing_users() {
                popup.set_user_view_height(height);
            } else {
                popup.set_list_view_height(height);
            }
        }
    }

    pub fn move_emoji_reaction_down(&mut self) {
        let reactions_len = self.filtered_emoji_reaction_items().len();
        if let Some(picker) = self.popups.emoji_reaction_picker_mut() {
            picker.selection.move_down(reactions_len);
        }
    }

    pub fn move_emoji_reaction_up(&mut self) {
        if let Some(picker) = self.popups.emoji_reaction_picker_mut() {
            picker.selection.move_up();
        }
    }

    pub fn set_emoji_reaction_picker_view_height(&mut self, height: usize) {
        let len = self
            .filtered_emoji_reaction_items_slice()
            .map(<[_]>::len)
            .unwrap_or(0);
        if let Some(picker) = self.popups.emoji_reaction_picker_mut() {
            picker.selection.set_view_height_and_sync(height, len);
        }
    }

    pub fn emoji_reaction_picker_scroll(&self) -> usize {
        self.popups
            .emoji_reaction_picker()
            .map(|picker| picker.selection.scroll())
            .unwrap_or(0)
    }

    pub fn selected_emoji_reaction_index_for_len(&self, len: usize) -> Option<usize> {
        self.popups
            .emoji_reaction_picker()
            .map(|picker| picker.selection.selected_for_len(len))
    }

    pub fn selected_emoji_reaction(&self) -> Option<EmojiReactionItem> {
        let items = self.filtered_emoji_reaction_items();
        let index = self.selected_emoji_reaction_index_for_len(items.len())?;
        items.get(index).cloned()
    }

    pub fn activate_selected_emoji_reaction(&mut self) -> Option<AppCommand> {
        let picker = self.popups.emoji_reaction_picker().cloned()?;
        let reaction = self.selected_emoji_reaction()?;
        let selected_message = self.selected_message_state().filter(|message| {
            message.channel_id == picker.channel_id && message.id == picker.message_id
        });
        if let Some(message) = selected_message
            && !self.can_add_reaction_to_message(message, &reaction.emoji)
        {
            self.close_emoji_reaction_picker();
            return None;
        }
        let already_reacted = selected_message.is_some_and(|message| {
            message.channel_id == picker.channel_id
                && message.id == picker.message_id
                && message
                    .reactions
                    .iter()
                    .any(|existing| existing.me && existing.emoji == reaction.emoji)
        });
        let command = if already_reacted {
            AppCommand::RemoveReaction {
                channel_id: picker.channel_id,
                message_id: picker.message_id,
                emoji: reaction.emoji,
            }
        } else {
            AppCommand::AddReaction {
                channel_id: picker.channel_id,
                message_id: picker.message_id,
                emoji: reaction.emoji,
            }
        };
        self.close_emoji_reaction_picker();
        Some(command)
    }

    pub fn activate_emoji_reaction_shortcut(&mut self, shortcut: char) -> Option<AppCommand> {
        let shortcut = shortcut.to_ascii_lowercase();
        let picker = self.popups.emoji_reaction_picker()?;
        let index = picker
            .filtered_items
            .iter()
            .enumerate()
            .position(|(index, _)| {
                self.options
                    .key_bindings()
                    .emoji_reaction_shortcut(&picker.filtered_items, index)
                    == Some(shortcut)
            })?;
        if let Some(picker) = self.popups.emoji_reaction_picker_mut() {
            picker.selection.select(index);
        }
        self.activate_selected_emoji_reaction()
    }

    pub fn start_emoji_reaction_filter(&mut self) {
        if let Some(picker) = self.popups.emoji_reaction_picker_mut() {
            picker.filter = Some(String::new());
            picker.filter_editing = true;
            picker.filtered_items = picker.items.clone();
            picker.selection.select(0);
        }
    }

    pub fn commit_emoji_reaction_filter(&mut self) {
        if let Some(picker) = self.popups.emoji_reaction_picker_mut() {
            picker.filter_editing = false;
        }
    }

    pub fn push_emoji_reaction_filter_char(&mut self, value: char) {
        if let Some(picker) = self.popups.emoji_reaction_picker_mut()
            && let Some(filter) = &mut picker.filter
        {
            filter.push(value);
            picker.filtered_items = filter_emoji_reaction_items_from_slice(&picker.items, filter);
            picker.selection.select(0);
        }
    }

    pub fn pop_emoji_reaction_filter_char(&mut self) {
        if let Some(picker) = self.popups.emoji_reaction_picker_mut()
            && let Some(filter) = &mut picker.filter
        {
            filter.pop();
            picker.filtered_items = filter_emoji_reaction_items_from_slice(&picker.items, filter);
            picker.selection.select(0);
        }
    }

    pub fn open_emoji_reaction_picker(&mut self) {
        if let Some(message) = self.selected_message_state() {
            if !self.can_open_reaction_picker(message) {
                return;
            }
            let guild_id = message
                .guild_id
                .or_else(|| self.selected_channel_guild_id());
            let existing_reactions = message
                .reactions
                .iter()
                .map(|reaction| reaction.emoji.clone())
                .collect::<Vec<_>>();
            let own_reactions = message
                .reactions
                .iter()
                .filter(|reaction| reaction.me)
                .map(|reaction| reaction.emoji.clone())
                .collect::<Vec<_>>();
            let items =
                self.build_emoji_reaction_picker_items(message, guild_id, &existing_reactions);
            self.popups.modal = Some(ModalPopup::EmojiReactionPicker(EmojiReactionPickerState {
                selection: Default::default(),
                filter: None,
                filter_editing: false,
                filtered_items: items.clone(),
                items,
                existing_reactions,
                own_reactions,
                guild_id,
                channel_id: message.channel_id,
                message_id: message.id,
            }));
        }
    }

    fn build_emoji_reaction_picker_items(
        &self,
        message: &MessageState,
        guild_id: Option<Id<GuildMarker>>,
        existing_reactions: &[ReactionEmoji],
    ) -> Vec<EmojiReactionItem> {
        if self.can_add_new_reaction_for_message(message) {
            prioritize_existing_reactions(
                self.emoji_reaction_items_for_guild(guild_id),
                existing_reactions,
            )
        } else {
            message
                .reactions
                .iter()
                .map(|reaction| EmojiReactionItem {
                    emoji: reaction.emoji.clone(),
                    label: reaction.emoji.status_label(),
                    is_pinned: self.reactions.pinned_emojis.contains(&reaction.emoji),
                })
                .collect()
        }
    }

    fn toggle_emoji_pin(&mut self, emoji: ReactionEmoji) {
        let pinned = &mut self.reactions.pinned_emojis;
        if let Some(position) = pinned.iter().position(|pinned| *pinned == emoji) {
            pinned.remove(position);
        } else {
            pinned.push_front(emoji);
        }
        self.options.ui_state_save_pending = true;
    }

    pub fn toggle_selected_emoji_reaction_pin(&mut self) {
        let Some(item) = self.selected_emoji_reaction() else {
            return;
        };
        self.toggle_emoji_pin(item.emoji);
        self.refresh_emoji_reaction_picker_items();
    }

    fn refresh_emoji_reaction_picker_items(&mut self) {
        let Some(picker) = self.popups.emoji_reaction_picker().cloned() else {
            return;
        };
        let items = {
            let Some(message) = self.selected_message_state().filter(|message| {
                message.channel_id == picker.channel_id && message.id == picker.message_id
            }) else {
                return;
            };
            self.build_emoji_reaction_picker_items(
                message,
                picker.guild_id,
                &picker.existing_reactions,
            )
        };
        let filtered_items = match picker.filter.as_deref() {
            Some(filter) if !filter.is_empty() => {
                filter_emoji_reaction_items_from_slice(&items, filter)
            }
            _ => items.clone(),
        };
        if let Some(picker) = self.popups.emoji_reaction_picker_mut() {
            picker.items = items;
            picker.filtered_items = filtered_items;
        }
    }

    fn picker_guild_id(&self) -> Option<Id<GuildMarker>> {
        self.popups
            .emoji_reaction_picker()
            .and_then(|picker| picker.guild_id)
            .or_else(|| {
                self.selected_message_state()
                    .and_then(|message| message.guild_id)
            })
            .or_else(|| self.selected_channel_guild_id())
    }
}

fn filter_emoji_reaction_items(
    items: Vec<EmojiReactionItem>,
    filter: &str,
) -> Vec<EmojiReactionItem> {
    filter_emoji_reaction_items_from_slice(&items, filter)
}

fn prioritize_existing_reactions(
    items: Vec<EmojiReactionItem>,
    existing_reactions: &[crate::discord::ReactionEmoji],
) -> Vec<EmojiReactionItem> {
    if existing_reactions.is_empty() {
        return items;
    }

    let mut prioritized = Vec::with_capacity(items.len());
    for existing in existing_reactions {
        if let Some(item) = items.iter().find(|item| &item.emoji == existing) {
            prioritized.push(item.clone());
        } else {
            prioritized.push(EmojiReactionItem {
                emoji: existing.clone(),
                label: existing.status_label(),
                is_pinned: false,
            });
        }
    }
    prioritized.extend(
        items
            .into_iter()
            .filter(|item| !existing_reactions.contains(&item.emoji)),
    );
    prioritized
}

fn filter_emoji_reaction_items_from_slice(
    items: &[EmojiReactionItem],
    filter: &str,
) -> Vec<EmojiReactionItem> {
    let filter = filter.trim();
    if filter.is_empty() {
        return items.to_vec();
    }

    // Pinned emojis always rank above unpinned ones, regardless of match
    // quality - same "pinned is always at the top" rule as pinned channels.
    let mut scored: Vec<(bool, usize, FuzzyScore, usize, EmojiReactionItem)> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            emoji_reaction_filter_score(item, filter).map(|score| {
                (
                    !item.is_pinned,
                    usize::from(emoji_reaction_is_remaining_unicode(item)),
                    score,
                    index,
                    item.clone(),
                )
            })
        })
        .collect();

    scored.sort_by_key(|(not_pinned, is_remaining_unicode, score, index, _)| {
        (*not_pinned, *is_remaining_unicode, *score, *index)
    });
    scored.into_iter().map(|(_, _, _, _, item)| item).collect()
}

fn emoji_reaction_is_remaining_unicode(item: &EmojiReactionItem) -> bool {
    matches!(&item.emoji, crate::discord::ReactionEmoji::Unicode(emoji) if !is_quick_unicode_emoji(emoji))
}

fn emoji_reaction_filter_score(item: &EmojiReactionItem, filter: &str) -> Option<FuzzyScore> {
    let label_score = fuzzy_text_score(&item.label, filter);
    let status_score = fuzzy_text_score(&item.emoji.status_label(), filter);
    match (label_score, status_score) {
        (Some(label), Some(status)) => Some(label.min(status)),
        (Some(label), None) => Some(label),
        (None, Some(status)) => Some(status),
        (None, None) => None,
    }
}
