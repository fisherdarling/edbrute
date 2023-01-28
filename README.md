# Ed tu, Brute?

A simple rust program for bruteforcing large ed25519 key pairs in parallel.

Run with
```
cargo run --release
```

The largest key found so far is printed in the format `hex-public-key <-> hex-private-key` like so:

```
fffff2906ebe5d556124625fadca871e1acfd5d8d732a30255276f7cfce29815 <-> c42ddfbe718dcaab9d0067c52d776c67f5c1231ccdde885ee6f51d42af8fecb5
```

Does not support check-pointing, yet.