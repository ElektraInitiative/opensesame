# Debugging

## Important Debugging Options

`kdb set user:/sw/libelektra/opensesame/#0/current/debug TODO`



## Trigger Watchdog

Usually you will disable the watchdog during development:

`kdb set user:/sw/libelektra/opensesame/#0/current/watchdog/enable "1"`

If you are logged in, simply do:

`sudo killall -9 opensesame; while true; do; echo b | sudo tee /dev/watchdog; sleep 1; done`

If you cannot login because it resets so fast, replace password and execute following to trigger the watchdog:

```
ssh olimex@haustuer cat \| sudo --prompt="" -S -- tee /dev/watchdog << EOF
password
b
EOF
```

Or if you need to stop (hanging) Opensesame first:

```
ssh olimex@haustuer cat \| sudo --prompt="" -S -- "sh -c 'killall -9 opensesame && tee /dev/watchdog'"  << EOF
password
b
EOF
```

## Corrupt History File

`zsh: corrupt history file /home/olimex/.zsh_history`

To fix use:

```
mv .zsh_history .zsh_history.broken && strings .zsh_history.broken > .zsh_history && fc -R .zsh_history
```
