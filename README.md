systemd-cron-next
=================

[![Travis](https://img.shields.io/travis/systemd-cron/systemd-cron-next.svg)](https://travis-ci.org/systemd-cron/systemd-cron-next)

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

## Usage

The generator runs on system boot and when the crontabs change.

The project includes simple `crontab` command equivalent, which behaves like standard crontab command
(and accepts the same main options).

After installation add executable scripts to the appropriate cron directory (e.g. `/etc/cron.daily`)
and enable systemd-cron:

    # systemctl daemon-reload
    # systemctl enable cron.target
    # systemctl start cron.target

The scripts should now be automatically run by systemd. See man:systemd.cron(7) for more information.

To further control cron jobs, use `cron.target` unit.

## Dependencies

* systemd ≥ 197
    * systemd ≥ 209, yearly timers
    * systemd ≥ 212, persistent timers
    * systemd ≥ 217, minutely, quarterly & semi-annually timers
* [run-parts][]
* /usr/sbin/sendmail (optional, evaluated at runtime)

[run-parts]: http://packages.qa.debian.org/d/debianutils.html "debianutils"

## Installation

If you are on [Archlinux][arch], install from [AUR][aur], otherwise see `PKGBUILD` file and
execute commands from `package()` sub.

[arch]: https://www.archlinux.org/
[aur]: https://aur.archlinux.org/packages/systemd-crontab-generator/

## Packaging

### Building

    $ ./configure
    $ make

You will need [Rust stable][rust-install] compiler (tested with 1.2.0) and cargo tool to build the project.

[rust-install]: http://www.rust-lang.org/install.html

### Staging

    $ make DESTDIR="$destdir" install

### Configuration

The `configure` script takes command line arguments to configure various details of the build. The following options
follow the standard GNU [installation directories][4]:

* `--prefix=<path>`
* `--bindir=<path>`
* `--confdir=<path>`
* `--datadir=<path>`
* `--libdir=<path>`
* `--statedir=<path>`
* `--mandir=<path>`
* `--docdir=<path>`

Other options include:

* `--unitdir=<path>` Path to systemd unit files.
  Default: `<libdir>/systemd/system`.
* `--runpaths=<path>` The path installations should use for the `run-parts` executable.
  Default: `<prefix>/bin/run-parts`.
* `--enable-boot[=yes|no]` Include support for the boot timer.
  Default: `yes`.
* `--enable-minutely[=yes|no]` Include support for the minutely timer. Requires systemd ≥ 217.
  Default: `no`.
* `--enable-hourly[=yes|no]` Include support for the hourly timer.
  Default: `yes`.
* `--enable-daily[=yes|no]` Include support for the daily timer.
  Default: `yes`.
* `--enable-weekly[=yes|no]` Include support for the weekly timer.
  Default: `yes`.
* `--enable-monthly[=yes|no]` Include support for the monthly timer.
  Default: `yes`.
* `--enable-quarterly[=yes|no]` Include support for the quarterly timer. Requires systemd ≥ 217.
  Default: `no`.
* `--enable-semi_annually[=yes|no]` Include support for the semi-annually timer. Requires systemd ≥ 217.
  Default: `no`.
* `--enable-yearly[=yes|no]` Include support for the yearly timer. Requires systemd ≥ 209.
  Default: `no`.
* `--enable-persistent[=yes|no]` Make timers [persistent][5]. Requires systemd ≥ 212.
  Default: `no`.

A typical configuration for the latest systemd would be:

    $ ./configure --prefix=/usr --confdir=/etc --enable-yearly --enable-persistent

If you only want the generator (you'll have to provide your own `/etc/crontab` to drive /etc/cron.daily/ etc...):

    $ ./configure --enable-boot=no --enable-hourly=no --enable-daily=no --enable-weekly=no --enable-month=no --enable-persistent --prefix=/usr --confdir=/etc

### Caveat

Your package should also run these extra commands before starting cron.target
to ensure that @reboot scripts doesn't trigger right away:

    # touch /run/crond.reboot
    # touch /run/crond.bootdir

## See Also

`systemd.cron(7)` or in source tree `man -l src/man/systemd.cron.7`

## Disclaimer

This is a beta product! Use at your own risk! I'm not responsible for any data losses,
time losses, money losses or any other failures due to use or misuse of this project!
I've run this product on my local server for several months without issues, but it does not
mean you will have no issues as well! Don't blame me for any crashes because of the product!
You were warned!

## License

The main part of a project is licensed under [MIT][].
Crontab man page is derived from [Vixie Cron][vixie] and licensed under *Paul-Vixie's-license*.

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
- © 2013-2014 Dwayne Bent
- © 2013 Dominik Peteler
- © 2014 Daniel Schaal <farbing@web.de>

Systemd crontab generator evolution, tooling and support:
- © 2014 Alexandre Detiste <alexandre@detiste.be>
- © 2014 Dwayne Bent

Original systemd crontab generator code in Python, Rust version:
- © 2014-2015 Konstantin Stepanov <me@kstep.me>

Systemd crontab generator man-page (*man/systemd-crontab-generator.8.in*):
- © 2014 Alexandre Detiste <alexandre@detiste.be>

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
