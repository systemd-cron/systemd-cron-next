PREFIX := /usr

target/release/systemd-crontab-generator target/release/boot-delay: src/*.rs src/bin/*.rs
	cargo build --release

target/debug/systemd-crontab-generator target/debug/boot-delay:
	cargo build

release: target/release/systemd-crontab-generator target/release/boot-delay

build: target/build/systemd-crontab-generator target/build/boot-delay

install: release
	install --mode=0755 --strip -D target/release/systemd-crontab-generator ${PREFIX}/lib/systemd/system-generators/systemd-crontab-generator
	install --mode=0755 --strip -D target/release/boot-delay ${PREFIX}/bin/boot-delay

.PHONY: build release install
