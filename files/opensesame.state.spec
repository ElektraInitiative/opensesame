# state specification for opensesame
#
# State is something that might be changed from opensesame itself but can
# also be changed by outside processes. After SIGHUP the state will automatically
# be applied.
#
# It is recommended to mount this file for development (then you get defaults+validation)
# simply run:
#
# kdb mount `pwd`/opensesame.state.spec spec:/state/libelektra/opensesame/#0/current ni
# kdb spec-mount /state/libelektra/opensesame/#0/current
#
# The second command needs to be re-executed when you change something in the spec.
#
# -_-
#  /

[]
mountpoint = opensesame.state
infos/plugins = quickdump shell execute/set=reload-opensesame

[alarm/fire]
description = Which room currently has a present alarm, if any. Can be triggered from both environment or sensors.

[environment/baseline]
description = internal state from CCS811 to be used in initalization. Is automatically saved every seven days or on shutdown.
