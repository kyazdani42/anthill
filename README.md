# Anthill - Maildir / Imap Sync Tool

## Notice

this is quite early stage, right now only syncing new mail is working (no purge/expunge).

## Why

- I had issues with isync and offlineimap that couldn't be solved easily.
- Every imap syncing tool is super slow, syncing one request at a time.
- This tool allows you to download mail quickly, using the power of multithreading combined with the speed of rust.
    - one account at a time
    - parallelize mailboxes (all mailboxes are syncing at the same time)
    - parallelize body fetch (mails are downloaded in parallel)

## Install

```
git clone https://git.sr.ht/~yazdan/anthill
cd anthill && cargo build --release
```

## Performance

> TODO, but right now i realized syncing my mails take 5/10 sec compared to 3/5 min with offlineimap/isync

## TODO

- 1 | remove out of date messages
- 2 | cli
- 3 | SSL/TLS
- 4 | password: gpg
- 5 | doc
- 6 | 2 way sync (expunge requests and cleanup mailboxes)
- 7 | OAUTH/XOAUTH (gmail)
