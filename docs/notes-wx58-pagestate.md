# Stage 3: the Settings pages read a prepared page

Stage 3 of `docs/notes-settings-extraction.md` §3, executed as crew task wx58.
Base revision `1dd08bd37` (*refactor(settings): Move the row kit into its own
crate (#224)*), so the row kit is already `spaceport-settings-window` and the
pages call it through the app's `settings_window::ui` façade, with `ui::Labels`
carrying the words the kit does not own.

Nothing moved out of the app. This is a change of shape: the values a page
draws are handed to it as one value, `PageState`, instead of being read off the
window inside the renderer.

## 1. Survey: what each page read from the window

Counted at `1dd08bd37` in `crates/spaceport-gpui/src/settings_window/pages/`
(2,369 production lines, 11 modules plus the dead `gitlab.rs`). `self.theme` and
`self.config` are reads of the window's own fields; `self.shared` is Herdr's
shared settings; `self.available`/`self.gate_shared`/`self.controls_shared_ready`
are window rules; the rest is window furniture.

| Page | Settings values read | Window rules | Other window state (stays) |
| --- | --- | --- | --- |
| `configuration.rs` | `self.theme` ×3 (`:46,:82,:112`), `self.shared` `:47` | `self.gate_shared` `:62` | `self.controls.local_path/local_file` `:59-60` |
| `fonts.rs` | `self.theme` ×7 (`:38,:65,:66,:70,:125,:126,:139`), `self.config` `:132` | — | `self.number_edit_for` `:59`, `self.control_face_size`, `self.font_button`, `self.active_font_face`, `self.render_font_specimen` |
| `general.rs` | `self.theme` ×5 (`:23,:260,:319,:335,:337`), `self.shared` `:29`, `self.config` (`:36,:68-69,:76-77,:87-88,:102-103,:108,:135,:158,:171-172,:254,:313,:315`) | `self.available` ×6 (`:82,:125,:144,:148,:167,:333`), `self.gate_shared` `:184`, `self.controls_shared_ready` `:24` | `self.persistence` ×6 (control enablement), `self.number_value`, `self.number_edit_for`, `self.launch_at_login_rows` |
| `indicators.rs` | `self.theme` ×2 (`:29,:88`), `self.shared` `:36`, `self.config` (`:30,:37,:89,:90`) | — | — |
| `language.rs` | `self.theme` ×2 (`:65,:66`), `self.config.language` `:50` | — | `self.persistence` ×2, `self.dropdown` `:71` |
| `notifications.rs` | `self.theme` ×6 (`:17,:157,:159,:179,:203,:206`), `self.shared` `:19`, `self.config` (`:20,:32,:151,:155,:200`) | `self.gate_shared` ×2 (`:43,:112`), `self.controls_shared_ready` `:18` | `self.source` `:33` (is the main window alive), `self.number_value`, `self.number_edit_for` ×2 |
| `sidebar.rs` | `self.theme` ×5 (`:63,:313,:341,:343,:377`), `self.config` (`:84,:101,:124,:142,:156,:171-172,:190-191,:200-201,:208-209,:216-217,:273,:301,:339,:369`; the reads inside `cx.listener` closures name `this.config` in the handler instead) | `self.available` `:95` | `self.persistence` ×2, `self.dropdown` `:318`, `self.hex_edit`, `self.host_rows`, `self.number_edit_for` |
| `status_bar.rs` | `self.theme` ×4 (`:50,:157,:179,:181`), `self.config` ×3 (`:51-53`) | — | — |
| `tabs.rs` | `self.theme` `:17`, `self.shared` `:19`, `self.config` (`:40,:49-50,:59-60,:107-108,:115-116,:128-129`) | `self.gate_shared` ×3 (`:71,:87,:139`), `self.available` `:55`, `self.controls_shared_ready` `:18` | `self.persistence` ×2 `:27`, `self.remote_history` `:153` |
| `theme.rs` | `self.theme` ×13 (`:66,:108,:122,:123,:149,:157,:180,:183,:207,:208,:230,:231,:260`), `self.config.theme` `:179`, `self.config.contrast` `:206` | `self.available` `:147` | `self.themes` ×3 (grid, library toggles, edited side), `self.edited_theme`, `self.drafted_theme` |
| `window.rs` | `self.theme` ×3 (`:22,:51,:52`), `self.config.terminal_padding.axis` `:49` | — | `self.number_value`, `self.number_edit_for`; the same field in `set_terminal_padding` `:82` is a handler |
| `pages.rs` (helpers) | `self.theme` `:77` | `self.shared_gate` `:38`, `self.shared` `:48` | `self.dropdown` `:80` |

Three things the survey settled:

- Every page reads the same three values (`config`, `shared`, `theme`); no page
  wants a different subset in a way that would justify a page-specific struct.
  `general.rs` is the widest with 17 `config` reads, `theme.rs` the narrowest with 2.
- Two window *rules* were being applied inside pages: `available(availability)`
  (the flags that hide a row, `search.rs:31`) and `shared_gate`/`gate_shared`
  (why a row editing Herdr's file is disabled, `pages.rs:45`). Both are pure
  functions of the values a page draws, so both belong with them.
- Everything else pages reach for is window furniture that stage 4 and up own:
  the open dropdown, the save queue's readiness, the theme browser, the remote
  history rows, the font/backdrop control state, the number edit and preview.
  None of it is drawn *from*; it is the window's own interaction state.

## 2. The shape

`SettingsWindow` keeps the fields. `PageState` (`page_state.rs`, 96 lines) is
the value one page draws, built once per page render by the window:

```rust
pub(in crate::settings_window) struct PageState<'a> {
    pub config: &'a Config,
    pub core_enabled: bool,
    pub drafted_theme: &'a str,
    pub shared: Option<&'a herdr_settings::Settings>,
    pub theme: &'a Theme,
}
```

- `config` and `theme` are borrows, not copies: a page paints the theme the
  window holds and the config it is editing, and a per-render `Theme` clone
  would either be a large copy or a stale one.
- `shared` is what Herdr's file said, or none; `Option<&'a Settings>` is the
  shape every page already matched on.
- `core_enabled` is the session's frozen Core opt-in (`core_feature::enabled`),
  read once where the page state is built because it is a GPUI global and the
  page must not reach for the app.
- `drafted_theme` is `SettingsWindow::drafted_theme()`: an unsaved draft, else
  the saved value. `available(FollowingSystemTheme)` is its only reader, and
  keeping the *name* here rather than a pre-computed `bool` keeps the rule in
  one place.

Two rules moved onto it, as the design asks:

- `PageState::available(availability)` — the old `SettingsWindow::available`,
  unchanged in meaning (`Core` now reads the prepared `core_enabled`).
- `PageState::shared_gate()` and `PageState::gate_shared(row)`. The reason text
  is built by the free `page_state::shared_gate(Option<&Settings>)`, so the one
  test that reads the gate off a window it holds (`tests/pages_behaviour.rs:230`)
  keeps its exact call through a `#[cfg(test)]` delegate.

One value, not a family: the alternative was eleven structs, each with the
exact field subset its page touches. That is a struct, a builder and a
signature change per page to save nothing — the borrow is the same borrow, and
a page that later needs one more field would have to grow the builder and every
caller. `BrowserHosts` is the precedent (`spaceport-browser/src/hosts.rs`):
one borrow bundle of the values the browser may read, built by the app.

### The borrow shape

Pages are still `impl SettingsWindow` methods for now (stage 6 moves them), and
every renderer already took `&self`, so the two-borrow form is the plain one:

```rust
let page = PageState::of(self, cx);          // one shared reborrow of self
self.render_general(&page, cx);              // and a second shared one
```

Handlers keep `&mut self`; only renderers take a page. `render_theme` is the
one renderer that was `&mut self` (`themes::render_theme_results` records the
grid's column count), so it draws the grid first and borrows the page after:

```rust
let results = self.render_theme_results(window, cx);
let page = PageState::of(self, cx);
```

Nothing between the two lines reads the grid state the results call writes, and
the divs are placed in their original order, so the page lays out identically.

## 3. The wiring

| Kind | Before | After |
| --- | --- | --- |
| Page renderers | `fn render_x(&self, cx) -> Div` | `fn render_x(&self, page: &PageState<'_>, cx) -> Div` |
| Page-local drawing helpers (`face_row`, `spacing_row`, `set_card`, `button_row`, `file_row`, `padding_row`, `theme_*_row`, `feature_rows`, `chat_width_controls`, `launch_at_login_rows`, …) | `&self` | take the same page |
| `preference_toggle` (`controls/preferences.rs`) | read `self.theme` | `preference_toggle(theme: &Theme, …)`, as the design names |
| `override_dropdown` (`pages.rs`) | read `self.theme` | `override_dropdown(theme: &Theme, …)`; it keeps `&self` for the open dropdown |
| `available`, `shared_gate`, `gate_shared` | `SettingsWindow` methods | `PageState` methods |
| `visible_results` (`search.rs`) | called `self.available` | builds one `PageState` and asks it |

`&self` stays on the helpers because they still build `cx.listener` closures and
read the window's interaction state (open dropdown, number edit, theme browser);
what they no longer do is reach through it for a value.

Words did not move: every `fl!` in a page is still the app's, and the
`SettingsWindow` still words the gate reason. `PageState` names no sentence —
its only words are the two the gate already had, which is why it can be lifted
into a crate the day the message lookup goes below the app (P1).

`PageState::of(window, cx)` is the only constructor: it reads the three fields,
the frozen Core flag and the drafted theme name. `search.rs`'s `visible_results`
and the three test call sites that used `view.available(...)` build the same
value, so there is one availability rule, not two.

No call site outside `crates/spaceport-gpui/src/settings_window/` changed.

## 4. What is left on the window, and for which stage

Pages still read window state that is *interaction*, not a drawn value. Each is
already named in the extraction note's stage 4-6 scopes:

| Still read | By | Where it goes |
| --- | --- | --- |
| `self.persistence.quitting()/closing()` (control enablement) | general, language, sidebar, tabs | stage 1's save policy, once the pages are below the app |
| `self.controls_shared_ready()` (shared rows editable) | general, notifications, tabs | the same save policy plus the window's error banner |
| `self.dropdown` (open override list) | pages.rs, language, sidebar | stage 4 (`inputs`) |
| `self.number_edit` / `self.number_preview` | every page with a number | stage 4 (`inputs`) |
| `self.themes` (grid, library toggles, edited side) | theme | stage 5 (theme browser) |
| `self.remote_history` | tabs | with the tabs page |
| `self.controls.*`, `self.source` | configuration, notifications | stage 4/6 |
| `self.hex_edit` | sidebar | with the swatches' owner |
| `self.config` in the handlers (`set_spacing`, `set_terminal_padding`) | sidebar, window | handlers, not renderers; they stay until the pages move |

## 5. Verification

```
cargo check  -p spaceport-gpui -p spaceport-settings-window --all-targets --features integration-test   # clean
cargo clippy -p spaceport-gpui -p spaceport-settings-window --all-targets --features integration-test   # clean
cargo fmt --all -- --check                                                                              # clean
cargo test   -p spaceport-gpui --bin spaceport settings                                                 # 280 passed, 0 failed
```

The stage's exit criterion — no renderer reads the window's own values:

```
$ grep -rn 'self\.\(config\|shared\|theme\|keymap\)\.' crates/spaceport-gpui/src/settings_window/pages
crates/spaceport-gpui/src/settings_window/pages/window.rs:92:        let sides = self.config.terminal_padding.axis(axis);
```

That one line is `set_terminal_padding`, the handler the criterion allows. No
`self.keymap` read existed in a page before or after. `tests/pages_behaviour.rs`
is unchanged and passes; `tests/catalog_rows.rs`, `tests/search.rs` and
`tests/theme_mode.rs` moved their three `view.available(...)` /
`view.drafted_is_light(...)` calls onto `PageState::of`, because that is where
those rules now live.
