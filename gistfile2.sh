#!/bin/sh

GENERATOR_DIR=/run/systemd/generator

rm -f $GENERATOR_DIR/cron-*.service
rm -f $GENERATOR_DIR/cron-*.timer
rm -f $GENERATOR_DIR/cron.target
/usr/lib/systemd/system-generators/systemd-crontab-generator $GENERATOR_DIR
systemctl daemon-reload
systemctl restart cron.target