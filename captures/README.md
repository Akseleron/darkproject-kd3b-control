# Reference packet captures

`raw/` is ignored by Git by default.

Fetch the public KD3B rev.2 captures from the OpenRGB support issue with:

```fish
./scripts/fetch-reference-captures.fish
```

Do not modify the raw captures. Generated normalized fixtures should go under `tests/fixtures/` or a future `captures/derived/` directory with provenance metadata.
