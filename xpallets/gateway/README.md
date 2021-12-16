# overview

This document tests sbtc cross-chain.



# Prepare Data

交易原文生成利用[`threshold_signature_api/musig2-dll/src/lib.rs`](https://github.com/chainx-org/threshold_signature_api/blob/main/musig2-dll/src/lib.rs)生成。

## 议会

成员

- Alice
- Bob
- Charlie

Runers  up

- FERDIE

## 信托

Alice

- 助记词

  ~~~rust
  let alice_phrase = "flame flock chunk trim modify raise rough client coin busy income smile";
  ~~~

- 热私钥

  ~~~rust
  let alice_priv_hot = r_get_my_privkey(alice_phrase, "hot");
  ~~~

  ~~~
  66e839545a7c8c6129e4b6d9b8238d213fd1ed8c4ed9f64a6b460f2e46883a96
  ~~~

- 冷私钥

  ~~~rust
  let alice_priv_cold = r_get_my_privkey(alice_phrase, "cold");
  ~~~

  ~~~
  cdd275f3068838a1fb678746f37039d7e3742aed617c5fd3d650f949e1b8d1ab
  ~~~

Bob

- 助记词

  ~~~rust
  let bob_phrase = "shrug argue supply evolve alarm caught swamp tissue hollow apology youth ethics";
  ~~~

- 热私钥

  ~~~rust
  let bob_priv_hot = r_get_my_privkey(bob_phrase, "hot");
  ~~~

  ~~~
  2a38c513a394dde94b24a711b1135b1e6d4af606e6cdbad382c65084f51be736
  ~~~

- 冷私钥

  ~~~rust
  let bob_priv_cold = r_get_my_privkey(bob_phrase, "cold");
  ~~~

  ~~~
  44f2801002a967014760ccbf52a8aab0217934d2ecb7c38db380f5338b9d2883
  ~~~

Charlie

- 助记词

  ~~~rust
  let charlie_phrase = "awesome beef hill broccoli strike poem rebel unique turn circle cool system";
  ~~~

- 热私钥

  ~~~rust
  let charlie_priv_hot = r_get_my_privkey(charlie_phrase, "hot");
  ~~~

  ~~~
  1bb59ec969f9f574f85f1cace19da8680fbb0e1f553ff9f62a0f021bbaefc8fb
  ~~~

- 冷私钥

  ~~~rust
  let charlie_priv_cold = r_get_my_privkey(charlie_phrase, "cold");
  ~~~

  ~~~
  247c02901adf3d0b4387ae48a8d90948efd2dd4edb7286171393701f0f40fc06
  ~~~

  