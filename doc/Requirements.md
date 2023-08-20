# Requirements

## Development

- use cases must exist for issues
- issues must exist for PRs

## Logging

- all logging should be done via Nextcloud chat

## Configuration

- all configuration specified via Elektra
- configuration should be as minimal as possible

## Robustness

- no panic during runtime (at startup is okay)
- on endless loops or failing hardware in modules, errors to Nextcloud should be sent and the module should get deactivated
- only if Tokio main loop gets stuck, the watchdog should trigger
