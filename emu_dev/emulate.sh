#!/bin/bash

socat -d -d pty,raw,echo=0,link=../fakettyACM0 exec:./payload.sh