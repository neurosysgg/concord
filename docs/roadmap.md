# Fork roadmap — rough plans (not yet built)

Running list of features planned but not implemented on this fork. Each
section is a starting point for a future session, not a spec — concrete
enough to pick up cold, hedged wherever something is genuinely unverified.

## Sending stickers (sticker picker, Phase 3)

Phase 1 (real sticker model) and Phase 2 (inline image rendering) are done —
see `[Unreleased]` in `CHANGELOG.md`. This is the rough scope for picking and
sending a sticker from the composer.

### What's missing today

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

### Rough shape of the work

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

### Open questions, not decided yet

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

### Known issue surfaced during Phase 2 (unrelated to sticker sending)

Scrolling a channel with rendered images (attachments, embeds, or now
stickers) causes visible redraw flicker. Not root-caused. Predates stickers;
they just added another inline-image source that makes it easier to trigger.
Likely lives in the general image-preview redraw/scroll path, not anything
sticker-specific — worth a dedicated investigation separate from sticker
work.

## `/gif` command and GIF favourites

### What's there today

`src/discord/builtin_commands.rs` already scaffolds `/gif` and `/tenor` as
recognized builtin slash commands — they parse correctly (require a search
query argument) but hit a hardcoded dead end:

```rust
BuiltinSlashCommand::Gif | BuiltinSlashCommand::Tenor => required_argument(argument)
    .map_or(BuiltinSlashCommandParse::Incomplete, |_| {
        BuiltinSlashCommandParse::Ready(BuiltinSlashCommandSubmit::Unsupported {
            message: "GIF slash commands are not supported in Concord yet".to_owned(),
        })
    }),
```

Nothing else GIF-related exists — no search integration, no favourites, no
picker. The only adjacent code is `gif_auto_play: Option<bool>` in
`src/discord/user_settings.rs:33` (a display setting for whether GIFs
autoplay — unrelated to search/favourites).

### Key open question before any implementation: where do search results come from

Discord's real client does **not** call Tenor directly with a
user-supplied API key — it proxies GIF search through Discord's own backend
(so search rides on the same account session concord already authenticates
with, no separate Tenor credential needed). The exact endpoint and request/
response shape are not confirmed from this codebase — nothing here has ever
talked to it. **This needs to be verified (e.g. via a network capture of the
official client's `/gif` search) before any implementation work starts.** If
that proxy turns out to be unavailable or unreliable to reverse-engineer,
the fallback is a direct Tenor API integration, which would require the user
to supply their own Tenor API key — a meaningfully worse experience and a
new kind of credential this app doesn't otherwise ask for.

### Good news on the rendering side

Sending a GIF is likely just sending its URL as message content — Discord's
own server-side link-embed generation turns a Tenor GIF link into a rich
embed automatically, the same way any pasted link becomes an embed today.
concord already renders embeds (`EmbedInfo`/`inline_preview_info`, extended
for stickers this session). If that holds, **no new message-rendering work
is needed** — the entire scope is the search-and-pick input experience, not
display.

### Rough shape of the work

1. **Verify the search data source** (see above) — blocks everything else.
2. **Search picker UI** — same family as the sticker picker and emoji
   reaction picker: type a query, get a scrollable/gridable list of results
   with thumbnails, select one. Live-search-as-you-type (debounced) rather
   than the emoji picker's fixed-catalog-then-filter model, since results
   come from a network call per query rather than a local list.
3. **Wire into `/gif <query>` and `/tenor <query>`** in
   `builtin_commands.rs` — replace the `Unsupported` branch, likely changing
   `BuiltinSlashCommandParse` to support an intermediate "open picker with
   this initial query" state rather than resolving straight to a
   `BuiltinSlashCommandSubmit`.
4. **Favourites** — Discord stores per-user GIF favourites in newer
   protobuf-encoded account settings (not the classic JSON settings object
   `src/discord/user_settings.rs` already parses), which is a different and
   less-explored part of Discord's API surface than anything this app
   currently touches. Needs its own research pass: confirm the read/write
   endpoint and payload shape before designing storage. If that turns out to
   be impractical to reverse-engineer, a **local-only favourites list**
   (persisted in `UiStateOptions`, same tier as pinned channels/emoji, not
   synced to Discord's account settings) is a reasonable fallback scope —
   works within concord, just doesn't show up favourited in the official
   client too.

### Open questions, not decided yet

- Confirm the actual search endpoint/auth (see above) before writing any
  code — this determines whether the feature is buildable as scoped at all.
- Favourites: real Discord-account-synced favourites vs. a simpler
  concord-local list, pending the protobuf-settings research above.
- Whether `/tenor` and `/gif` should behave identically (Discord's real
  client treats them as synonyms) or diverge.
