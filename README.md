# Ed tu, Brute?

A simple rust program for bruteforcing large ed25519 key pairs in parallel. Supports checkpointing.

Run with
```
cargo run --release
```

The public keys of recently found keys are printed to the terminal along with the keys/s and 
total number of keys checked.

```
bruteforcing with 8 threads
[2023-01-28 13:33:18] fffe15b7c1a222c2b2c56bf468c0e6fc284f21e9835349902821095e87b823cf
[2023-01-28 13:33:18] fffebca1e58e7d14543ef8952cccbcb31caf9ed9349147d3a3796f27161f939b
[2023-01-28 13:33:18] ffffc70d509dbb98affab37f1c303dfa5b3af3b67c117291bab4cd63b803b55f
[2023-01-28 13:33:18] fffff95243db7df61b61ae00a59b20fb5f3160559101304d99a8e39d5c789a02

⠠ [00:00:06] 384538.65 keys/s, 2,621,400
  largest: fffff95243db7df61b61ae00a59b20fb5f3160559101304d99a8e39d5c789a02
```

Inside the generated `checkpoint.log` file, you'll find a list of found large key pairs.

When the program starts, it loads the checkpoint file and uses the largest key found in
that file as the starting point for brute forcing. The checkpoint file can be used across runs.