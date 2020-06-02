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

- 1 | remove out of date messages
- 2 | cli
- 3 | SSL/TLS
- 4 | password: gpg
- 5 | doc
- 6 | 2 way sync (expunge requests and cleanup mailboxes)
- 7 | OAUTH/XOAUTH (gmail)
