# Anthill - Maildir / Imap Sync Tool

## Why

- I had issues with isync and offlineimap that couldn't be solved easily.
- Emails should be downloaded asynchronously.
- Email sync should be fast.

## Install

with cargo:
```shell
cargo install anthill
```

git:
```shell
git clone git@git.sr.ht:kiyan42/anthill
cd anthill && cargo build --release
```

## TODO

- [ ] SSL/TLS
- [ ] password: gpg
- [ ] 2 way sync (expunge requests and cleanup mailboxes)

