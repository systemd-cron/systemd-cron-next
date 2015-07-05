target/release/systemd-crontab-generator target/release/boot-delay: src/*.rs src/bin/*.rs
	cargo build --release
	strip $@

target/debug/systemd-crontab-generator target/debug/boot-delay:
	cargo build

release: target/release/systemd-crontab-generator target/release/boot-delay

build: target/build/systemd-crontab-generator target/build/boot-delay

.PHONY: build release
