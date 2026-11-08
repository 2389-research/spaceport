# Root README: development status and ownership

The root README previously introduced Spaceport but then carried 500 lines of inherited Sheprd install commands, release links, screenshots, and feature claims. Those instructions named upstream package and update owners, a Herdr 1 startup route, and product surfaces already removed or still being removed. A reader could mistake them for a Spaceport install guide.

The root README now states the fork's development status, points to the authoritative specification, task/capability/acceptance records, names separately installed Core and open gates, gives one read-only build check, and preserves the fork's origin and license links. It does not claim native or release qualification. Git history retains the upstream README for reference. This is a documentation correction, not removal of the remaining source or package routes.

Review: the linked files and the pinned toolchain/temporary binary names were checked against this branch; `git diff --check` passed. No app code, native UI, package, updater, installed app, or personal Core was exercised for this document change. T12 remains blocked by G-RELEASE and G-PLATFORM; A42–A44 remain not_run. Inherited release scripts still need independent identity and package tests before publication.
