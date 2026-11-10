#!/usr/bin/env python3
"""ABOUTME: Require a complete, successful main-branch CI run for a release commit.
ABOUTME: Select the repository and trusted CI actor explicitly for fork releases.
"""
import argparse
import json
import os
import re
import subprocess
import sys

REPO = "sheprd-labs/sheprd"
ACTOR = "penso"
LOGIN = r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,37}[A-Za-z0-9])?"
# Job names are the API contract with ci.yml. Missing/renamed jobs fail closed.
# Windows packaging is path-selected and also runs in the release workflow;
# the macOS release builds run on pull requests only, since the release
# workflow builds those targets itself.
REQUIRED_JOBS = frozenset({
    "Workflow Security", "Commit messages", "Format, lint, and test",
    *(f"Format, lint, and test ({runner}, {features})" for runner in
      ("macos-15", "ubuntu-24.04", "ubuntu-24.04-arm") for features in ("default", "all")),
    *(f"Windows check ({runner}, {features})" for runner in ("windows-2025", "windows-11-arm")
      for features in ("default", "all")),
    *(f"Windows native GUI ({runner})" for runner in ("windows-2025", "windows-11-arm")),
    *(f"Release ({target})" for target in
      ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu")),
})


class CiError(Exception):
    pass


def gh(*args, **kwargs):
    return subprocess.run(["gh", *args], env={**os.environ, "GH_HOST": "github.com"},
                          text=True, **kwargs)


def api(path, *, repo=REPO, paginate=False):
    options = ["--paginate", "--slurp"] if paginate else []
    return json.loads(gh("api", f"repos/{repo}/{path}", *options,
                         check=True, stdout=subprocess.PIPE).stdout)


def eligible(run, sha, actor=ACTOR):
    return (run["head_sha"] == sha and run["head_branch"] == "main"
            and run["event"] in ("push", "workflow_dispatch")
            and run["actor"]["login"] == actor
            and run["triggering_actor"]["login"] == actor)


def missing_jobs(jobs):
    successful = {job["name"] for job in jobs
                  if job["status"] == "completed" and job["conclusion"] == "success"}
    return REQUIRED_JOBS - successful


def require_ci(sha, wait=False, *, repo=REPO, actor=ACTOR):
    runs = api(f"actions/workflows/ci.yml/runs?head_sha={sha}&branch=main&per_page=20", repo=repo)
    candidates = [run for run in runs["workflow_runs"] if eligible(run, sha, actor)]
    for run in candidates:
        if run["status"] != "completed" or run["conclusion"] != "success":
            continue
        pages = api(f"actions/runs/{run['id']}/jobs?filter=latest&per_page=100", repo=repo, paginate=True)
        missing = missing_jobs(job for page in pages for job in page["jobs"])
        if not missing:
            print(f"Full CI passed: https://github.com/{repo}/actions/runs/{run['id']}")
            return
        print(f"CI run {run['id']} lacks successful jobs: {', '.join(sorted(missing))}",
              file=sys.stderr)
    if wait:
        pending = next((run for run in candidates if run["status"] != "completed"), None)
        if pending:
            print(f"Waiting for CI: https://github.com/{repo}/actions/runs/{pending['id']}", flush=True)
            # The release's ci-gate job can watch for over an hour on the
            # workflow token's hourly API budget; poll every 30s, not 3s.
            gh("run", "watch", str(pending["id"]), "--repo", repo, "--exit-status", "--interval", "30",
               check=False, stdout=subprocess.DEVNULL)
            # Re-query jobs even when watch succeeds: the run may have skipped them.
            return require_ci(sha, repo=repo, actor=actor)
    raise CiError(f"No full successful CI run for {sha}. Run "
                  f"`gh workflow run ci.yml --repo {repo} --ref main`, "
                  "wait for it, then retry the release.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sha")
    parser.add_argument("--wait", action="store_true")
    parser.add_argument("--repo", default=REPO, help="GitHub owner/repository whose CI must pass")
    parser.add_argument("--actor", default=ACTOR, help="GitHub login required for both CI actors")
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.sha):
        parser.error("sha must be a full lowercase commit SHA")
    if (not re.fullmatch(rf"{LOGIN}/[A-Za-z0-9_.-]+", args.repo)
            or args.repo.split("/")[-1] in (".", "..")):
        parser.error("repo must be a GitHub owner/repository")
    if not re.fullmatch(LOGIN, args.actor):
        parser.error("actor must be a GitHub login")
    try:
        require_ci(args.sha, args.wait, repo=args.repo, actor=args.actor)
    except (CiError, OSError, subprocess.CalledProcessError, ValueError, KeyError) as error:
        print(f"CI release gate failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
