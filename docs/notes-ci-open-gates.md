# CI actor gates opened to any collaborator

The release and CI workflows used to name one login: every job's `if:` required
`github.actor == 'harperreed' && github.triggering_actor == 'harperreed'`, the
`macos-preview` job conditions repeated it, and
`scripts/release/check-ci.py` hard-coded `ACTOR = "harperreed"`. The owner asked
that any collaborator be able to run main CI and cut a preview, so those
comparisons are gone:

- `.github/workflows/ci.yml`, `supply-chain.yml` and `cache-macos.yml` keep only
  the repository, event and `refs/heads/main` requirements.
- `.github/workflows/macos-preview.yml` keeps the main-branch dispatch or `v*`
  tag route and no longer passes `--actor` to its `ci-gate` step.
- `check-ci.py` defaults `ACTOR` to `None`, which accepts whoever GitHub reports
  for a run; `--actor LOGIN` still pins both actors for a fork release. Its
  `main()` now skips the login check when no `--actor` was given, which it must,
  or the default path would crash on `None`.

What still gates a run, none of it by login: the repository must be
`2389-research/spaceport`, main CI must be complete and successful on the exact
commit, the version must be unused, the signing and publishing jobs re-fetch the
tag and refuse it if it moved, signing takes only that run's checksummed
artifact, and only the publishing job holds `contents: write`.

## The compensating gate: what GitHub reports (2026-11-25)

`gh api repos/2389-research/spaceport/environments/macos-preview`:

```json
{"name": "macos-preview", "can_admins_bypass": true,
 "protection_rules": [{"id": 71985591, "type": "branch_policy"}],
 "deployment_branch_policy": {"protected_branches": false, "custom_branch_policies": true}}
```

`gh api repos/2389-research/spaceport/environments/macos-preview/deployment-branch-policies`
returns two policies: branch `main` and tag `v*`.

**Finding: the `macos-preview` environment has no required reviewers.** Its one
protection rule is the branch policy above, so GitHub releases the environment's
six signing secrets to any run that arrives on `main` or a `v*` tag with no
second person's approval. `can_admins_bypass: true` also lets an admin skip
whatever rules the environment does hold.

Those secrets are the only copy of the signing material: the repository has no
repository-level Actions secrets, and these six are environment-scoped —
`MACOS_CERTIFICATE_P12_BASE64`, `MACOS_CERTIFICATE_PASSWORD`,
`APPLE_API_PRIVATE_KEY`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER_ID` and
`SPARKLE_PRIVATE_KEY`.

The other half of the boundary is repository write access:

| Check | Reported state (2026-11-25) |
| --- | --- |
| `gh api repos/2389-research/spaceport/branches/main/protection` | `404 Not Found` — no legacy branch protection |
| `gh api repos/2389-research/spaceport/rulesets` | `[]` — no rulesets either |
| `gh api repos/2389-research/spaceport/collaborators` | 7 collaborators: 4 admin (`harperreed`, `detour1999`, `sugi-chan`, `sophiealula`), 3 write (`clintecker`, `ibips`, `hlee1400`) |

So the effective policy today is: **any of those seven logins can push a `v*`
tag or dispatch `macos-preview.yml` on main, and the signed, notarized artifact
is published with no review**. Before this change the workflow itself refused
every other login, so the actor gate was the only thing standing in for a
reviewer. `.github/CODEOWNERS` claims `* @harperreed`, but a code-owner rule is
advisory without a branch protection or ruleset that requires it, and neither
exists.

This is the finding to flag: signing authority widened to every collaborator
while the environment provides no required reviewer. It is a GitHub settings
change, not a repository change, so these files cannot close it.

## Closing it, when the owner wants to

1. Add required reviewers to the `macos-preview` environment (Settings >
   Environments > macos-preview), enough that the login pushing the tag is not
   the only person who can release the secrets. GitHub does not count a
   deployer's own review.
2. Turn off `can_admins_bypass` so an admin cannot sign alone.
3. Keep the environment's branch policy on `main` and `v*`, and give main a
   ruleset that requires the `Format, lint, and test` gate plus a review, so a
   tag can only be pushed from a commit that already passed.

Each of those is outside `scripts/release` and outside this branch; record the
result here or in the release README when it lands.

## Verification for this change

Run in the main checkout on this branch, 2026-11-25:

| Check | Result |
| --- | --- |
| `actionlint .github/workflows/*.yml` | clean |
| `python3 -m unittest discover -s scripts/release/tests` | 404 tests, 10 skipped, one failure: `test_changelog.RealChangelogTests.test_history_renders_conventional_types_into_keep_a_changelog_groups`, which fails here for every branch because tags such as `v20261108.1` are not ancestors of `HEAD` in this clone |
| `python3 scripts/release/test-release-security.py` | 3 tests, clean |
| `git diff --check` | clean |

The tests changed with the policy rather than around it: `test_ci_policy.py`
now proves an actor-independent condition for every job and that no workflow
names `github.actor`, `test_check_ci.py` proves an unknown login qualifies and
the default `main()` path reaches the API, and the preview tests assert the
absence of `--actor` in the workflow.

Not run: a live tag push or dispatch. The first real preview exercises the new
route, and nothing in CI can rehearse GitHub's environment behaviour.

## Left open

`gotchas.md`'s dated 2026-11-10 entry under D66 still says the repository move
"keeps trusted actor `harperreed`", and the D66 record in
`docs/spaceport/plan/decisions.json` carries the same clause. Both are decision
records that this change outruns; amending or superseding D66 is the owner's
call, not a tidying edit here.
