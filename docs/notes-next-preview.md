# Next preview — changelog draft

> **Draft. Do not publish this as it stands.** We have not cut or published the
> next preview. This is the working draft of its release body, built from
> `v20261125.1..a8fbdb5668` on 25 November 2026 (last preview: `v20261125.1`).
> The generated list at the bottom is what `git-cliff` makes of that range. The
> plain-language part above it is ours to edit before the release.

## What you will notice

Short version: this preview looks and works like the last one. No screen moved,
no shortcut changed, and no command answers differently. Most of the work was
moving rules into their own crates so we can test them without opening a window.

### Shortcuts work exactly as before

We moved the code that turns a key press into an app command out of the app and
into its own crate. Nothing about your keys changed:

- Every default shortcut, every `[keybindings]` entry, and every Herdr key your
  window already used resolves the same way.
- `spaceport keybinds` and `spaceport keybinds check` print the same answers.
  Herdr's custom commands are still checked against the same rules.
- Herdr's `[keys]` table, its prefixes, its `[keys.indexed]` digits and the keys
  that move the workspace picker are read as they were.

The reason to move it: we can now run the key rules as tests on their own. The
risky part of a key change used to need a whole window.

### Fixed on the phone

The iPhone, iPad and Android clients build separately and are not in this
preview. Two fixes from sharing rules with them are worth recording anyway:

- A screenshot upload to a real macOS host no longer fails. The phone used to
  expect `/tmp`, and a macOS host prints `/private/tmp`. Both clients now build
  the same staging script, so the phone accepts the path a host really prints.
- A search on the phone now waits for the view to settle before it sends, and
  tries once more after a refusal. Before, it asked against whatever frame it
  had and reported a timeout for a search it never sent.

- A full queue no longer hangs up the phone's session. When the desktop's
  daemon queue was busy, the phone used to end the connection and give up on
  the focus request. Now it waits and tries again, exactly like the desktop.
- Terminal links and scroll position survive the trip to the phone, so the
  next cell change can't quietly drop them again.

### One rule for scrolling

The phone had its own copy of the rule that turns finger and wheel motion into
the daemon's mouse contract. Both clients now use the same rule. Desktop
scrolling did not change. The phone sends the same numbers it did before.

### Easier to check what we kept

- The README's download link now points at the last preview, `v20261125.1`,
  instead of one five previews back. Its feature reachability snapshot was
  re-derived against the current tree, so what we kept and what we removed can be
  checked at the revision it names.
- The README also says plainly which two leftovers of removed features still
  exist, and that the mobile clients are a separate workspace that reads the
  desktop crates by path.

## Was anything fixed for the desktop?

No user-visible fix this time. We kept the release honest rather than dressing up
a refactor as a fix: if a screen, a shortcut or an exit status had changed, we
would say so here.

## The generated changelog

This is `git-cliff` output for `v20261125.1..a8fbdb5668`, unchanged. Commit
types the release body hides (`ci`, `chore`, `test`, `style`, `build`) are
already dropped by `cliff.toml`, so the two CI commits and one chore commit in
that range do not appear.

```markdown
### Changed
- Share one wheel rule between the desktop and the phone (#238)
- [keymap] Move the daemon key table into spaceport-keymap (#239)
- [client] Share the upload staging and search pacing rules with the phone (#241)
- [keymap] Move the keymap's resolution into spaceport-keymap (#243)

### Documentation
- [spaceport] Record that stage 3 folded into stage 2
- Refresh the README and re-derive the reachability snapshot (#246)
```

## Before this goes out

- Re-run `git-cliff --config cliff.toml --unreleased --tag vYYYYMMDD.1` once the
  tag is chosen, and paste that output over the block above. The tag date decides
  the version line.
- Check that nothing user-visible landed between `a8fbdb5668` and the release
  commit. If a `fix` or `feat` lands, it belongs in "What you will notice".
- Keep the summary short. A quiet release says so once and stops.
