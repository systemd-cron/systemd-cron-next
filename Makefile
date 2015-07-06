PREFIX := /usr

target/release/boot-delay: src/bin/boot-delay.rs
	cargo build --release

target/release/mail-on-failure: src/bin/mail-on-failure.rs
	cargo build --release

target/release/systemd-crontab-generator: src/*.rs
	cargo build --release

target/debug/boot-delay: src/bin/boot-delay.rs
	cargo build

target/debug/mail-on-failure: src/bin/mail-on-failure.rs
	cargo build

target/debug/systemd-crontab-generator:
	cargo build

release: target/release/systemd-crontab-generator target/release/boot-delay target/release/mail-on-failure

build: target/debug/systemd-crontab-generator target/debug/boot-delay target/debug/mail-on-failure

install: release
	install --mode=0755 --strip -D target/release/systemd-crontab-generator ${PREFIX}/lib/systemd/system-generators/systemd-crontab-generator
	install --mode=0755 --strip -D target/release/mail-on-failure ${PREFIX}/bin/mail-on-failure
	install --mode=0755 --strip -D target/release/boot-delay ${PREFIX}/bin/boot-delay
	install --mode=0644 -D units/cron.target ${PREFIX}/lib/systemd/system/cron.target

.PHONY: build release install
