use super::*;
use crate::discord::GuildFolder;

#[test]
fn channel_switcher_groups_channels_and_filters_by_fuzzy_name() {
    let mut state = DashboardState::new();
    state.push_event(AppEvent::ChannelUpsert(ChannelInfo {
        last_message_id: Some(Id::new(100)),
        ..dm_channel_info(Id::new(40), "alice")
    }));
    state.push_event(guild_create_event(
        Id::new(1),
        "guild",
        vec![
            category_channel_info(Id::new(1), Id::new(10), "Text", 0),
            child_text_channel_info(Id::new(1), Id::new(11), Id::new(10), "general", 0),
            child_text_channel_info(Id::new(1), Id::new(12), Id::new(10), "random", 1),
        ],
    ));

    state.push_event(AppEvent::ReadStateInit {
        entries: vec![read_state_info(Id::new(40), Some(Id::new(100)), 0)],
    });

    state.open_channel_switcher();
    let all_items = state.channel_switcher_items();
    assert_eq!(all_items[0].group_label, "Direct Messages");
    assert_eq!(all_items[1].group_label, "guild");
    assert_eq!(all_items[1].parent_label.as_deref(), Some("Text"));

    state.push_event(AppEvent::ChannelUpsert(child_text_channel_info(
        Id::new(1),
        Id::new(13),
        Id::new(10),
        "general-new",
        2,
    )));

    for ch in "gnrl".chars() {
        state.push_channel_switcher_char(ch);
    }
    let filtered = state.channel_switcher_items();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].channel_id, Id::new(11));

    state.close_channel_switcher();
    state.open_channel_switcher();
    for ch in "gnrl".chars() {
        state.push_channel_switcher_char(ch);
    }
    let filtered: Vec<Id<ChannelMarker>> = state
        .channel_switcher_items()
        .into_iter()
        .map(|item| item.channel_id)
        .collect();
    assert!(filtered.contains(&Id::new(11)));
    assert!(filtered.contains(&Id::new(13)));
}

#[test]
fn channel_switcher_includes_threads_and_forums_with_type_icons() {
    let guild_id = Id::new(1);
    let general_id = Id::new(11);
    let forum_id = Id::new(20);
    let thread_id = Id::new(31);
    let mut state = state_with_channel_tree();
    state.push_event(AppEvent::ChannelUpsert(forum_channel_info(
        guild_id, forum_id,
    )));
    // A joined, non-archived thread under a text channel must appear.
    state.push_event(AppEvent::ChannelUpsert(ChannelInfo {
        current_user_joined_thread: Some(true),
        ..thread_channel_info(guild_id, general_id, thread_id, "a thread")
    }));

    state.open_channel_switcher();
    let items = state.channel_switcher_items();
    let label = |id: Id<ChannelMarker>| {
        items
            .iter()
            .find(|item| item.channel_id == id)
            .map(|item| item.channel_label.as_str())
    };

    assert_eq!(label(general_id), Some("# general"));
    assert_eq!(label(forum_id), Some("📝 announcements"));
    assert_eq!(label(thread_id), Some("🧵 a thread"));

    let thread = items
        .iter()
        .find(|item| item.channel_id == thread_id)
        .expect("joined thread should be listed");
    assert_eq!(
        thread.parent_label.as_deref(),
        Some("Text Channels / general")
    );
}

#[test]
fn channel_switcher_items_carry_unread_metadata() {
    let mut state = DashboardState::new();
    state.push_event(AppEvent::ChannelUpsert(ChannelInfo {
        last_message_id: Some(Id::new(100)),
        ..dm_channel_info(Id::new(40), "new")
    }));
    state.push_event(AppEvent::ReadStateInit {
        entries: vec![read_state_info(Id::new(40), Some(Id::new(90)), 0)],
    });
    state.open_channel_switcher();

    let items = state.channel_switcher_items();

    assert_eq!(items[0].channel_id, Id::new(40));
    assert_eq!(items[0].unread, ChannelUnreadState::Unread);
}

#[test]
fn channel_switcher_query_prefers_channel_name_before_context() {
    let mut state = DashboardState::new();
    state.push_event(guild_create_event(
        Id::new(1),
        "acme",
        vec![positioned_text_channel_info(
            Id::new(1),
            Id::new(11),
            "general",
            0,
        )],
    ));
    state.push_event(guild_create_event(
        Id::new(2),
        "other",
        vec![positioned_text_channel_info(
            Id::new(2),
            Id::new(21),
            "acme-chat",
            0,
        )],
    ));

    state.open_channel_switcher();
    for ch in "acme".chars() {
        state.push_channel_switcher_char(ch);
    }
    let filtered: Vec<Id<ChannelMarker>> = state
        .channel_switcher_items()
        .into_iter()
        .map(|item| item.channel_id)
        .collect();

    assert_eq!(filtered, vec![Id::new(21), Id::new(11)]);
}

#[test]
fn pane_filters_prioritize_prefix_matches() {
    let mut state = DashboardState::new();
    state.push_event(guild_create_event(
        Id::new(1),
        "Rust Programming language",
        vec![
            positioned_text_channel_info(Id::new(1), Id::new(11), "Rust Programming language", 0),
            positioned_text_channel_info(Id::new(1), Id::new(12), "MINECRAFT", 1),
        ],
    ));
    state.push_event(guild_create_event(Id::new(2), "MINECRAFT", Vec::new()));

    state.open_guild_pane_filter();
    for ch in "mi".chars() {
        state.push_guild_pane_filter_char(ch);
    }
    let guild_ids: Vec<Id<GuildMarker>> = state
        .guild_pane_filtered_entries()
        .into_iter()
        .filter_map(|entry| match entry {
            GuildPaneEntry::Guild { state, .. } => Some(state.id),
            _ => None,
        })
        .collect();
    assert_eq!(guild_ids, vec![Id::new(2), Id::new(1)]);

    state.activate_guild(ActiveGuildScope::Guild(Id::new(1)));
    state.open_channel_pane_filter();
    for ch in "mi".chars() {
        state.push_channel_pane_filter_char(ch);
    }
    let channel_ids: Vec<Id<ChannelMarker>> = state
        .channel_pane_filtered_entries()
        .into_iter()
        .filter_map(|entry| match entry {
            ChannelPaneEntry::Channel { state, .. } => Some(state.id),
            _ => None,
        })
        .collect();
    assert_eq!(channel_ids, vec![Id::new(12), Id::new(11)]);

    let mut state = DashboardState::new();
    state.push_event(guild_create_event(Id::new(1), "Alpha One", Vec::new()));
    state.push_event(guild_create_event(Id::new(2), "Alpha Two", Vec::new()));
    state.push_event(user_settings_update(vec![GuildFolder {
        id: Some(42),
        name: Some("folder".to_owned()),
        color: None,
        guild_ids: vec![Id::new(2), Id::new(1)],
    }]));

    state.open_guild_pane_filter();
    for ch in "al".chars() {
        state.push_guild_pane_filter_char(ch);
    }
    let guild_ids: Vec<Id<GuildMarker>> = state
        .guild_pane_filtered_entries()
        .into_iter()
        .filter_map(|entry| match entry {
            GuildPaneEntry::Guild { state, .. } => Some(state.id),
            _ => None,
        })
        .collect();
    assert_eq!(guild_ids, vec![Id::new(2), Id::new(1)]);

    let mut state = DashboardState::new();
    state.push_event(guild_create_event(
        Id::new(1),
        "guild",
        vec![
            positioned_text_channel_info(Id::new(1), Id::new(11), "Alpha One", 1),
            positioned_text_channel_info(Id::new(1), Id::new(12), "Alpha Two", 0),
        ],
    ));
    state.activate_guild(ActiveGuildScope::Guild(Id::new(1)));
    state.open_channel_pane_filter();
    for ch in "al".chars() {
        state.push_channel_pane_filter_char(ch);
    }
    let channel_ids: Vec<Id<ChannelMarker>> = state
        .channel_pane_filtered_entries()
        .into_iter()
        .filter_map(|entry| match entry {
            ChannelPaneEntry::Channel { state, .. } => Some(state.id),
            _ => None,
        })
        .collect();
    assert_eq!(channel_ids, vec![Id::new(12), Id::new(11)]);
}

#[test]
fn channel_switcher_lists_recent_channels_first() {
    let mut state = DashboardState::new();
    state.push_event(guild_create_event(
        Id::new(1),
        "guild",
        vec![
            ChannelInfo {
                last_message_id: Some(Id::new(101)),
                ..positioned_text_channel_info(Id::new(1), Id::new(11), "alerts", 0)
            },
            positioned_text_channel_info(Id::new(1), Id::new(12), "quiet", 1),
        ],
    ));

    state.activate_channel(Id::new(11));
    state.activate_channel(Id::new(12));
    state.activate_channel(Id::new(11));
    state.open_channel_switcher();
    let items = state.channel_switcher_items();

    assert_eq!(items[0].group_label, "Recent Channels");
    assert_eq!(items[0].channel_id, Id::new(12));
    assert_eq!(items[0].parent_label.as_deref(), Some("guild"));
    assert_eq!(
        items
            .iter()
            .filter(|item| {
                item.group_label == "Recent Channels" && item.channel_id == Id::new(12)
            })
            .count(),
        1
    );
    assert!(
        items
            .iter()
            .skip(1)
            .any(|item| { item.group_label == "guild" && item.channel_id == Id::new(11) })
    );
    assert!(
        items
            .iter()
            .any(|item| { item.group_label == "guild" && item.channel_id == Id::new(12) })
    );
}

#[test]
fn channel_switcher_query_matches_display_prefixes() {
    let mut state = DashboardState::new();
    state.push_event(AppEvent::ChannelUpsert(dm_channel_info(
        Id::new(40),
        "new-dm",
    )));
    state.push_event(guild_create_event(
        Id::new(1),
        "guild",
        vec![positioned_text_channel_info(
            Id::new(1),
            Id::new(11),
            "new-text",
            0,
        )],
    ));

    state.open_channel_switcher();
    for ch in "#new".chars() {
        state.push_channel_switcher_char(ch);
    }
    let filtered: Vec<Id<ChannelMarker>> = state
        .channel_switcher_items()
        .into_iter()
        .map(|item| item.channel_id)
        .collect();
    assert_eq!(filtered, vec![Id::new(11)]);

    state.close_channel_switcher();
    state.open_channel_switcher();
    for ch in "@new".chars() {
        state.push_channel_switcher_char(ch);
    }
    let filtered: Vec<Id<ChannelMarker>> = state
        .channel_switcher_items()
        .into_iter()
        .map(|item| item.channel_id)
        .collect();
    assert_eq!(filtered, vec![Id::new(40)]);
}

#[test]
fn channel_switcher_toggle_pin_adds_and_removes_pinned_group() {
    let mut state = DashboardState::new();
    state.push_event(guild_create_event(
        Id::new(1),
        "guild",
        vec![
            positioned_text_channel_info(Id::new(1), Id::new(11), "alerts", 0),
            positioned_text_channel_info(Id::new(1), Id::new(12), "quiet", 1),
        ],
    ));

    state.open_channel_switcher();
    let row = state
        .channel_switcher_items()
        .iter()
        .position(|item| item.channel_id == Id::new(12))
        .expect("quiet channel listed");
    state.select_channel_switcher_item(row);
    state.toggle_selected_channel_switcher_pin();

    let items = state.channel_switcher_items();
    assert_eq!(items[0].group_label, "Pinned Channels");
    assert_eq!(items[0].channel_id, Id::new(12));
    assert!(items[0].is_pinned);
    // A pinned channel shows up exactly once, in the Pinned Channels group,
    // not duplicated in its normal guild listing.
    assert_eq!(
        items
            .iter()
            .filter(|item| item.channel_id == Id::new(12))
            .count(),
        1
    );

    let row = items
        .iter()
        .position(|item| item.group_label == "Pinned Channels")
        .expect("pinned entry listed");
    state.select_channel_switcher_item(row);
    state.toggle_selected_channel_switcher_pin();

    let items = state.channel_switcher_items();
    assert!(
        !items
            .iter()
            .any(|item| item.group_label == "Pinned Channels")
    );
    assert!(items.iter().all(|item| !item.is_pinned));
    // Unpinning brings it back into its normal guild listing.
    assert!(
        items
            .iter()
            .any(|item| item.group_label == "guild" && item.channel_id == Id::new(12))
    );
}

#[test]
fn channel_switcher_pinned_group_sits_above_recent_and_boosts_search_ranking() {
    let mut state = DashboardState::new();
    state.push_event(guild_create_event(
        Id::new(1),
        "guild",
        vec![
            positioned_text_channel_info(Id::new(1), Id::new(11), "alerts", 0),
            positioned_text_channel_info(Id::new(1), Id::new(12), "quiet", 1),
            positioned_text_channel_info(Id::new(1), Id::new(13), "lounge", 2),
        ],
    ));
    // 12 ends up recent (not active); 13 is active and excluded from Recent.
    state.activate_channel(Id::new(12));
    state.activate_channel(Id::new(13));

    state.open_channel_switcher();
    let row = state
        .channel_switcher_items()
        .iter()
        .position(|item| item.channel_id == Id::new(11))
        .expect("alerts channel listed");
    state.select_channel_switcher_item(row);
    state.toggle_selected_channel_switcher_pin();

    let items = state.channel_switcher_items();
    assert_eq!(items[0].group_label, "Pinned Channels");
    assert_eq!(items[0].channel_id, Id::new(11));
    assert_eq!(items[1].group_label, "Recent Channels");
    assert_eq!(items[1].channel_id, Id::new(12));

    for ch in "l".chars() {
        state.push_channel_switcher_char(ch);
    }
    let filtered: Vec<Id<ChannelMarker>> = state
        .channel_switcher_items()
        .into_iter()
        .map(|item| item.channel_id)
        .collect();
    assert_eq!(filtered[0], Id::new(11));
}

#[test]
fn channel_switcher_pin_hard_overrides_search_ranking() {
    let mut state = DashboardState::new();
    state.push_event(guild_create_event(
        Id::new(1),
        "guild",
        vec![
            positioned_text_channel_info(Id::new(1), Id::new(11), "office", 0),
            positioned_text_channel_info(Id::new(1), Id::new(12), "offtopic", 1),
        ],
    ));

    state.open_channel_switcher();
    for ch in "off".chars() {
        state.push_channel_switcher_char(ch);
    }
    // Unpinned: the tighter "office" match outranks "offtopic".
    let filtered: Vec<Id<ChannelMarker>> = state
        .channel_switcher_items()
        .into_iter()
        .map(|item| item.channel_id)
        .collect();
    assert_eq!(filtered[0], Id::new(11));
    state.close_channel_switcher();

    state.open_channel_switcher();
    let row = state
        .channel_switcher_items()
        .iter()
        .position(|item| item.channel_id == Id::new(12))
        .expect("offtopic channel listed");
    state.select_channel_switcher_item(row);
    state.toggle_selected_channel_switcher_pin();

    for ch in "off".chars() {
        state.push_channel_switcher_char(ch);
    }
    // Pinned "offtopic" now ranks first despite the weaker match.
    let filtered: Vec<Id<ChannelMarker>> = state
        .channel_switcher_items()
        .into_iter()
        .map(|item| item.channel_id)
        .collect();
    assert_eq!(filtered[0], Id::new(12));
}

#[test]
fn channel_switcher_pinned_group_shows_unread_channels_first() {
    let mut state = DashboardState::new();
    state.push_event(guild_create_event(
        Id::new(1),
        "guild",
        vec![
            ChannelInfo {
                last_message_id: Some(Id::new(100)),
                ..positioned_text_channel_info(Id::new(1), Id::new(11), "alerts", 0)
            },
            positioned_text_channel_info(Id::new(1), Id::new(12), "quiet", 1),
        ],
    ));
    state.push_event(AppEvent::ReadStateInit {
        entries: vec![read_state_info(Id::new(11), Some(Id::new(90)), 0)],
    });

    state.open_channel_switcher();
    // Pin "quiet" (read) before "alerts" (unread), so pin order alone would
    // put "quiet" on top - unread priority should still win.
    let row = state
        .channel_switcher_items()
        .iter()
        .position(|item| item.channel_id == Id::new(12))
        .expect("quiet channel listed");
    state.select_channel_switcher_item(row);
    state.toggle_selected_channel_switcher_pin();

    let row = state
        .channel_switcher_items()
        .iter()
        .position(|item| item.channel_id == Id::new(11))
        .expect("alerts channel listed");
    state.select_channel_switcher_item(row);
    state.toggle_selected_channel_switcher_pin();

    let pinned: Vec<Id<ChannelMarker>> = state
        .channel_switcher_items()
        .into_iter()
        .filter(|item| item.group_label == "Pinned Channels")
        .map(|item| item.channel_id)
        .collect();
    assert_eq!(pinned, vec![Id::new(11), Id::new(12)]);
}

#[test]
fn channel_switcher_pin_persists_through_ui_state_round_trip() {
    let mut state = state_with_channel_tree();

    state.open_channel_switcher();
    let row = state
        .channel_switcher_items()
        .iter()
        .position(|item| item.channel_id == Id::new(11))
        .expect("general channel listed");
    state.select_channel_switcher_item(row);
    state.toggle_selected_channel_switcher_pin();

    let ui_state = state
        .take_ui_state_save_request()
        .expect("pin toggle should request a UI state save");
    assert_eq!(ui_state.pinned_channel_ids, vec![Id::new(11)]);

    let restored = DashboardState::new_with_options(
        DisplayOptions::default(),
        Default::default(),
        Default::default(),
        NotificationOptions::default(),
        VoiceOptions::default(),
        Default::default(),
        ui_state,
    );
    assert!(
        restored
            .navigation
            .channels
            .pinned_channel_ids
            .contains(&Id::new(11))
    );
}

#[test]
fn channel_switcher_query_edits_at_cursor() {
    let mut state = DashboardState::new();
    state.open_channel_switcher();
    for ch in "raXndom".chars() {
        state.push_channel_switcher_char(ch);
    }

    for _ in 0..5 {
        state.move_channel_switcher_query_cursor_left();
    }
    state.move_channel_switcher_query_cursor_right();
    state.pop_channel_switcher_char();

    assert_eq!(state.channel_switcher_query(), Some("random"));
    assert_eq!(
        state.channel_switcher_query_cursor_byte_index(),
        Some("ra".len())
    );
}

#[test]
fn channel_switcher_query_deletes_grapheme_before_cursor() {
    let mut state = DashboardState::new();
    state.open_channel_switcher();
    for ch in "e\u{301}x".chars() {
        state.push_channel_switcher_char(ch);
    }

    state.move_channel_switcher_query_cursor_left();
    state.pop_channel_switcher_char();

    assert_eq!(state.channel_switcher_query(), Some("x"));
    assert_eq!(state.channel_switcher_query_cursor_byte_index(), Some(0));
}
