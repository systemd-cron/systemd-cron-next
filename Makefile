PREFIX := /usr

target/release/%: src/bin/%.rs
	cargo build --release

target/debug/%: src/bin/%.rs
	cargo build

target/release/systemd-crontab-generator: src/*.rs
	cargo build --release

target/debug/systemd-crontab-generator: src/*.rs
	cargo build

release: target/release/systemd-crontab-generator target/release/boot-delay target/release/mail-on-failure

build: target/debug/systemd-crontab-generator target/debug/boot-delay target/debug/mail-on-failure

install: release
	find target/release -maxdepth 1 -executable -type f -not -name systemd-crontab-generator -execdir \
	    install --mode=0755 --strip -D {} ${PREFIX}/usr/bin/{} \;
	install --mode=0755 --strip -D target/release/systemd-crontab-generator ${PREFIX}/lib/systemd/system-generators/systemd-crontab-generator
	install --mode=0644 -D units/cron.target ${PREFIX}/lib/systemd/system/cron.target

.PHONY: build release install

.SUFFIXES: .rs
