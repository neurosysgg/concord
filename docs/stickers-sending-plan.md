# Sending stickers — rough plan (not yet built)

Phase 1 (real sticker model) and Phase 2 (inline image rendering) are done —
see the `[Unreleased]` section of `CHANGELOG.md`. This is a rough scope for
Phase 3: picking and sending a sticker from the composer. Nothing here is
implemented; it's a starting point for the next session, not a spec.

## What's missing today

- No sticker catalog anywhere. Discord sends guild stickers the same way it
  sends guild emoji (a `stickers` array on `GUILD_CREATE`, plus a
  `GUILD_STICKERS_UPDATE` gateway event mirroring `GUILD_EMOJIS_UPDATE`) —
  concord parses neither. Mirror the existing emoji path:
  - `src/discord/gateway/parser/guilds.rs` — parses `emojis` off `GUILD_CREATE`
    (~line 79) and off `GUILD_EMOJIS_UPDATE` (~line 186/218). Add the same for
    `stickers`.
  - `src/discord/events.rs:201` — `AppEvent::GuildEmojisUpdate`. Add
    `AppEvent::GuildStickersUpdate`.
  - `src/discord/state.rs:277` — applies the emoji event into the guild
    cache. Add the sticker equivalent, plus cache accessors mirroring
    `custom_emojis_for_guild`/`all_custom_emojis()`.
  - New `GuildStickerInfo` type (id, name, format_type, guild_id, tags for
    search) mirroring `CustomEmojiInfo`. Can likely reuse `StickerFormatType`
    from Phase 1 directly.
- No REST support for sending one. `message_request_body_with_tts` in
  `src/discord/rest/messages.rs:350` builds the create-message JSON body
  (content, nonce, reply reference, allowed_mentions) but never sets
  `sticker_ids`. Discord allows up to 3 sticker IDs per message, sent
  alongside optional text content — this is a small, additive change once a
  sticker is actually selected.
- No composer UI to pick one. `ComposerUiState` in
  `src/tui/state/composer/state.rs:71` already has the right shape to extend:
  `pending_composer_attachments: Vec<MessageAttachmentUpload>` is the existing
  precedent for "things queued to send with the message" (a
  `pending_composer_stickers: Vec<StickerItemInfo>` — capped at 3 — would sit
  right next to it), and `ComposerPickerState`/`ActiveComposerPicker` already
  hosts the mention/command autocomplete pickers as an enum of picker modes —
  a sticker picker is a natural new `ActiveComposerPicker::Sticker` variant
  rather than a bolt-on popup system.

## Rough shape of the work

1. **Guild sticker catalog** (gateway parsing + cache) — foundation, no UI
   change. Mirrors the emoji-update path file-for-file.
2. **Picker UI** — closest existing analog is the emoji reaction picker
   (`src/tui/state/popups/reactions.rs` / `src/tui/ui/popups/reactions.rs`):
   fuzzy filter, list of catalog items, thumbnail preview. Phase 2's
   `inline_previews()` pipeline (or the smaller `EmojiImageCache`-style inline
   glyph, whichever the picker's layout ends up needing) is already available
   for showing a sticker thumbnail in the list.
3. **Queue-before-send** — selecting a sticker in the picker adds it to
   `pending_composer_stickers` rather than sending immediately (unlike the
   reaction picker, which acts on selection right away) — closer to how a
   pending attachment gets queued, shown, and can be removed before hitting
   send. Needs its own small chip/indicator in the composer area.
4. **REST send** — thread `pending_composer_stickers` IDs into
   `message_request_body_with_tts` as `sticker_ids`, clear the pending list on
   successful send.

## Open questions to resolve before starting, not decided yet

- **Scope of the catalog for v1**: current guild's stickers only, or also
  Nitro cross-server access from other guilds (mirrors the existing
  `current_user_has_nitro()` gating already built for foreign custom emoji in
  `emoji_reaction_items_for_guild`)? Recommend guild-only for v1.
- **Discord's global standard sticker packs** (thousands of stickers via a
  separate `/sticker-packs` REST catalog, unrelated to any guild) are
  explicitly out of scope — that's a browse-a-large-catalog problem on its
  own, not a small addition.
- **Single vs multi-select**: Discord allows up to 3 stickers per message.
  Recommend single-select for v1 (simpler picker, matches the reaction
  picker's select-and-act flow) with multi-select as a clear, isolated
  follow-up.
- Exact keybinding to open the picker while composing — no slot claimed yet.

## Known issue surfaced during Phase 2 (unrelated to this plan)

Scrolling a channel with rendered images (attachments, embeds, or now
stickers) causes visible redraw flicker. Not root-caused. Predates stickers;
they just added another inline-image source that makes it easier to trigger.
Likely lives in the general image-preview redraw/scroll path, not anything
sticker-specific — worth a dedicated investigation separate from sticker
work.
