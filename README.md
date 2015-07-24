systemd-cron-next
=========================

## What's it?

This is a compatibility layer for crontab-to-systemd timers framework. It works by parsing
crontab and anacrontab files from usual places like `/etc/crontab` and `/var/spool/cron`
and generating systemd timers and services. You can use `cron.target` as a single control
point for the generated units.

It's intented to be drop-in replacement for all cron implementations.

## Rationale

The crontab generator for systemd (implemented in C) was already [published][] on maillist,
but was later [declined][] by Lennart:

> I am not convinced this is a really good idea. From my perspective at
least it appears that we should much rather just convert the crontabs
and that's it. Unlike sysv init scripts the number of crontab in use (at
least on Fedora) is relatively small, and very rare in third-party
packages.
>
> Lennart

While I'm generally agree with him, I'm not totally convinced it's always convenient to herd a bunch of
separate `*.timer` and `*.service` files. I find convenient to have all jobs schedule in one single file,
and it's more obvious to see relations in jobs scheduling if you have several jobs on sight at once.

All things considered, I think people should have an alternative here, so I decided to support
the idea of systemd crontab generator. Though the original C implementation of crontab parser
from maillist is very incomplete: it doesn't support monotonic schedules (like `@daily` or `@yearly`),
it can't parse comments and environment variable settings, etc.

Hence I decided to create **systemd-crontab-generator**.

[published]: http://lists.freedesktop.org/archives/systemd-devel/2013-August/012591.html
[declined]: http://lists.freedesktop.org/archives/systemd-devel/2013-September/013120.html

## History

I'm not very good in C, so C implementation (while it's recommended for generators) whould take
me ages to write, so at first I used Python for proof-of-concept implementation.
Then my small home project was noticed by **[@systemd-cron][]** project and eventually was merged
into it and evolved thanks to **Alexanre Detiste**, **Dwayne Bent** and others.

Still I beared in mind the image of the project's future: rewrite it in systems language.
Python, being VM-based scripting language is not the best choice for system service:
it's slow (the slowest systemd generator ever, actually), have problems with multithreading,
requires a lot of hacks like setgid/setuid C helper to implement crontab, etc.

Meanwhile the [Rust][] systems language, I liked very much from the very beginning,
[reached its 1.0][announce], so I decided to grab the moment and rewrite everything in Rust.

The current version you are starring at is meant to be a successor of **[@systemd-cron][]**
project, fully rewritten in Rust from ground up, while preserving experience, systemd unit
templates and main algorithms and solutions polished in Python version by **[@systemd-cron][]** team.

[Rust]: http://www.rust-lang.org
[announce]: http://blog.rust-lang.org/2015/05/15/Rust-1.0.html

## Installation

If you are on [Archlinux][arch], install from [AUR][aur], otherwise see `PKGBUILD` file and
execute commands from `package()` sub.

[arch]: https://www.archlinux.org/
[aur]: https://aur.archlinux.org/packages/systemd-crontab-generator/

## Usage

The generator runs on system boot and when the crontabs change.

The project includes simple `crontab` command equivalent, which behaves like standard crontab command
(and accepts the same main options).

To control cron jobs, use `cron.target`, e.g. to start and enable cron after installation:

    # systemctl enable cron.target
    # systemctl start cron.target

## Disclaimer

This is a beta product! Use at your own risk! I'm not responsible for any data losses,
time losses, money losses or any other failures due to use or misuse of this project!
I've run this product on my local server for several months without issues, but it does not
mean you will have no issues as well! Don't blame me for any crashes because of the product!
You were warned!

## License

The main part of a project is licensed under [MIT][].
Crontab man page is derived from [Vixie Cron][vixie] and licensed under *Paul-Vixie's-license*.
Don't forget to attribute if you derive from the work!

[vixie]: https://wiki.gentoo.org/wiki/Cron#vixie-cron
[MIT]: http://opensource.org/licenses/MIT

## Contribution

You are most welcome to post [bugs][issues] and [PRs][pulls]!
Also check out [comments][] in AUR for current news about Arch package status.

[issues]: https://github.com/kstep/systemd-crontab-generator/issues
[pulls]: https://github.com/kstep/systemd-crontab-generator/pulls
[comments]: https://aur.archlinux.org/packages/systemd-crontab-generator/?comments=all

## Copyright

Original **[@systemd-cron][]** project:
- © 2013 Dwayne Bent
- © 2013 Dominik Peteler
- © 2014 Daniel Schaal <farbing@web.de>

Systemd crontab generator evolution, tooling and support:
- © 2014 Alexandre Detiste <alexandre@detiste.be>
- © 2014 Dwayne Bent

Original systemd crontab generator code in Python, Rust version:
- © 2014-2015 Konstantin Stepanov <me@kstep.me>

Crontab man-page (*man/crontab.5.in*):
- © 1988, 1990, 1993, 1994, Paul Vixie <paul@vix.com>
- © 1994, Ian Jackson <ian@davenant.greenend.org.uk>
- © 1996-2005, Steve Greenland <stevegr@debian.org>
- © 2005-2006, 2008-2012, Javier Fernández-Sanguino Peña <jfs@debian.org>
- © 2010-2011, 2014 Christian Kastner <debian@kvr.at>
- Numerous contributions via the Debian BTS copyright their respective authors

Debian packaging:
- © 2013 Shawn Landden <shawn@churchofgit.com>
- © 2014 Alexandre Detiste <alexandre@detiste.be>

[@systemd-cron]: https://github.com/systemd-cron/systemd-cron
