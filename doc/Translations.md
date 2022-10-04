## Get Translation Strings

cargo install xtr
~/.cargo/bin/xtr src/main.rs -o opensesame.pot

## Start a new Translation

msginit

## Update an existing Translation

msgmerge -U de_AT.po opensesame.pot

## After Translation

msgfmt de_AT.po
mv messages.mo files/opensesame.mo
