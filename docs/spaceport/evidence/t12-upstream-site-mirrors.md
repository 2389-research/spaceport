# T12 inherited site mirror repair

The upstream Sheprd site remains in this source tree and is not a qualified
Spaceport release site. Its test suite had a stale assertion that opened the
deleted desktop phone dialog, then asserted a TestFlight link in the root
README that no longer exists. The site invite check now covers only the
historical site's pages. The existing mirror tests still require identical
installers and `llms.txt` across `site/`, `website/site/`, and the Herdr UI
worker's `public/` directory.

The first site run failed on the missing phone dialog. After removing that
stale app assertion, it exposed mismatched Windows installer and `llms.txt`
mirrors. The second run failed on the worker's copies. All four mirrors now
match the current checked-in `site/` source, which no longer registers the
inherited `sheprd://` or `herdr-gpui://` links in its Windows installer.
`python3 -m unittest discover -s site/tests -p 'test_*.py' -q` passed 24/24.

No site or installer was published or run. The inherited Sheprd domain,
repository, and download identity are still present in these files. They
cannot be used as Spaceport release identity; T12 and A42 remain blocked/not
run pending owner-controlled production identifiers and native release QA.
