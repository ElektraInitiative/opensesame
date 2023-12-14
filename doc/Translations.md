# Get Translation Strings

```bash
cargo install xtr
~/.cargo/bin/xtr src/lib.rs -o lang/opensesame.pot
```
# Start a new Translation
```bash
cd lang && msginit
```
# Update an existing Translation
```bash
msgmerge -U lang/de_AT.po lang/opensesame.pot
```
# After Translation
```bash
msgfmt lang/de_AT.po
mv messages.mo files/opensesame.mo
```
