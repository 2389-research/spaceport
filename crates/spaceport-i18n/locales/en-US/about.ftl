# The About box, worded as a demoscene NFO file: every row is one line of a
# release file, its field name padded out to the same width with stops.
# Project names arrive as placeables and stay verbatim, as do the copyright
# holders' names, the license and the addresses. The addresses are placeables
# too, so Fluent isolates them and a row reads right in either direction.

# $version is the app's version, such as "0.42.0".
about-version = version...: { $version }
about-publisher = group.....: { $name } by { $publisher }
# $url is the address of this project's source.
about-repository = code......: { $url }
about-modifications = (c).......: { $name } modifications { $copyright } { $publisher }
about-original-copyright = original..: { $name } work { $copyright } { $author }.
# $url is the address of the repository's third-party notice.
about-notices = notices...: { $url }
about-license = license...: { $license }
about-summary = info......: Native workspace for terminals and AI agents, powered by Herdr and herdr-core.
# The project names are kept verbatim in every locale.
about-upstream-credits = credits...: Built on work by the { $sheprd } and { $herdr } teams.
about-unaffiliated = note......: An independent fork, not affiliated with or endorsed by the { $herdr } or { $sheprd } teams.
# A team's home page: $name stays verbatim and $url is its address.
about-upstream-team = www.......: { $name } { $url }
# A worktree build's branch and pull request: $branch and $number are its own.
about-branch = branch....: { $branch }
about-pull-request = pr........: #{ $number }
# The greeting that rides the sine wave across the top of the About box's
# stage, as a demo scene announces itself. $publisher and $name stay verbatim.
about-marquee = { $publisher } presents { $name }
about-ok = [ OK ]
