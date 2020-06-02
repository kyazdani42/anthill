# Anthill - Maildir / Imap Sync Tool

## Notice

this is very early stage, completely untested, lots of lacking features.

## Why

- I had issues with isync and offlineimap that couldn't be solved easily.
- Mailbox should be fetch in parallel.
- Syncing mailboxes should be fast.

## Install

```
git clone https://git.sr.ht/~yazdan/anthill
cd anthill && cargo build --release
```

## TODO

- 1 | multiple body download at the same time
- 2 | remove out of date messages
- 3 | cli
- 4 | SSL/TLS
- 5 | password: gpg
- 6 | doc
- 7 | 2 way sync (expunge requests and cleanup mailboxes)
- 8 | OAUTH/XOAUTH (gmail)
