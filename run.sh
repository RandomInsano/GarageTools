#!/bin/sh

for io in `seq 132 139`; do
	echo $io > /sys/class/gpio/export
	echo out > /sys/class/gpio/gpio$io/direction
	chmod 666 /sys/class/gpio/gpio$io/value
done

target/debug/garage-tools

