# Anthill - Maildir / Imap Sync Tool

## Notice

this is very early stage, completely untested, lots of lacking features.

## Why

- I had issues with isync and offlineimap that couldn't be solved easily.
- Emails should be downloaded asynchronously.
- Email sync should be fast.

## Install

git:
```
git clone https://git.sr.ht/~yazdan/anthill
cd anthill && cargo build --release
```

## TODO

- [ ] cli
- [ ] remove out of date messages
- [ ] multithreading and request parallelism
- [ ] SSL/TLS
- [ ] password: gpg
- [ ] 2 way sync (expunge requests and cleanup mailboxes)

