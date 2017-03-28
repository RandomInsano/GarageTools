#!/bin/sh

for io in `seq 132 139`; do
	echo $io > /sys/class/gpio/export
done

target/debug/garage-tools

