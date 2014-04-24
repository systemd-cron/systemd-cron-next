systemd-crontab-generator
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

While I'm generally agree with him, I'm not totally convinced it's always convinient to herd a bunch of
separate `*.timer` and `*.service` files. I find convinient to have all jobs schedule in one single file,
and it's more obvious to see relations in jobs scheduling if you have several jobs on sight at once.

All things considered, I think people should have an alternative here, so I decided to support
the idea of systemd crontab generator. Though the original C implementation of crontab parser
from maillist is very incomplete: it doesn't support monotonic schedules (like `@daily` or `@yearly`),
it can't parse comments and environment variable settings, etc.

I'm not very good in C, so C implementation (while it's recommended for generators) whould take
me ages to write, so I used Python for proof-of-concept implementation. And here comes
**systemd-crontab-generator**.

[published]: http://lists.freedesktop.org/archives/systemd-devel/2013-August/012591.html
[declined]: http://lists.freedesktop.org/archives/systemd-devel/2013-September/013120.html

## Installation

If you are on [Archlinux][arch], install from [AUR][aur], otherwise see `PKGBUILD` file and
execute commands from `package()` sub.

[arch]: https://www.archlinux.org/
[aur]: https://aur.archlinux.org/packages/systemd-crontab-generator/

## Usage

The generator runs on system boot. If you change your crontabs in runtime, run `systemd-crontab-update`
script as root (`sudo systemd-crontab-update`) to regenerate systemd units and reload them.

The project includes simple `crontab` command equivalent, which behaves like standard crontab command
(and accepts the same main options), and runs `systemd-crontab-update` command after user crontab file
update. Note, though, the systemd-crontab-update requires superuser priviledges, so `crontab` tries
to run it under `sudo`, so if you are not allowed to run `systemd-crontab-update` via `/etc/sudoers`
file, you can't update crontab timers.

To control cron jobs, use `cron.target`, e.g. to start and enable cron after installation:

    # systemctl enable cron.target
    # systemctl start cron.target

## Disclaimer

This is not a production-ready project! Use at your own risk! I'm not responsible for any data losses,
time losses, money losses or any other failures due to usage of this project! You were warned!

## License

The project is licensed under [GPLv3](https://www.gnu.org/copyleft/gpl.html).
Don't forget to attribute if you derive from my work!

## Contribution

You are most welcome to post [bugs][issues] and [PRs][pulls]!
Also check out [comments][] in AUR for current news about Arch package status.

[issues]: https://github.com/kstep/systemd-crontab-generator/issues
[pulls]: https://github.com/kstep/systemd-crontab-generator/pulls
[comments]: https://aur.archlinux.org/packages/systemd-crontab-generator/?comments=all
