# Validator

This module is used together with buttons.

## Config

Is a map of:

- names to be printed **to**
- codes to be accepted.

An example in ansible/playbook.yaml:

```yaml
        validator:
           'uns mit 12-34': "14, 12, 13, 15, 11, 3, 7, 15"
```


And one in TOML syntax:

```toml
validator."uns mit 12-34" = "14, 12, 13, 15, 11, 3, 7, 15"
```
