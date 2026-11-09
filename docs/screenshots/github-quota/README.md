# GitHub quota warning

Native macOS screenshots for the rate-limit indicator and account dialog.

These captures predate the Sheprd rename and retain the names and artwork of
the original builds. The former GitHub quota fixture and its `mockup` feature
are no longer present in Spaceport.

- `before-account-dialog.png`: the installed Herdr `20261102.2` account
  dialog, cropped to exclude the surrounding workspace. It shows sign-in
  status but no quota information.
- `after-{dark,light}-{narrow,wide}.png`: this branch's isolated native
  fixture, with a synthetic exhausted allowance of 5,000 GraphQL points and
  a 29-minute retry delay. The narrow window is 360 points wide; the wide
  window is 1,200 points wide. Themes are Catppuccin Mocha and Latte.

The historical fixture did not contact GitHub or read saved credentials.
