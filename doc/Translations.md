## Get Translation Strings

cargo install xtr
~/.cargo/bin/xtr src/main.rs -o lang/opensesame.pot

## Start a new Translation

cd lang && msginit

## Update an existing Translation

msgmerge -U lang/de_AT.po lang/opensesame.pot

## After Translation

msgfmt lang/de_AT.po
mv messages.mo files/opensesame.mo
