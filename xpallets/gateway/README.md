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
  let alice_priv_hot = r_get_my_privkey(alice_phrase, "hot").unwrap();
  ~~~

  ~~~
  66e839545a7c8c6129e4b6d9b8238d213fd1ed8c4ed9f64a6b460f2e46883a96
  ~~~

- 冷私钥

  ~~~rust
  let alice_priv_cold = r_get_my_privkey(alice_phrase, "cold").unwrap();
  ~~~

  ~~~
  cdd275f3068838a1fb678746f37039d7e3742aed617c5fd3d650f949e1b8d1ab
  ~~~

- 热公钥

  ~~~rust
  let alice_pub_hot = r_get_my_pubkey(alice_priv_hot).unwrap();   
  ~~~

  ~~~
  02926877f1a4c5e348c32ab6307799f8ac6836bf60a2c3a38e56a759cabe8f0187
  ~~~

- 冷公钥

  ~~~rust
  let alice_pub_cold = r_get_my_pubkey(alice_priv_cold).unwrap(); 
  ~~~

  ~~~
  039392e66cb126ce7116a4dacd2682ddd80721f951b106818b03fea3e836713d12
  ~~~

Bob

- 助记词

  ~~~rust
  let bob_phrase = "shrug argue supply evolve alarm caught swamp tissue hollow apology youth ethics";
  ~~~

- 热私钥

  ~~~rust
  let bob_priv_hot = r_get_my_privkey(bob_phrase, "hot").unwrap();
  ~~~

  ~~~
  2a38c513a394dde94b24a711b1135b1e6d4af606e6cdbad382c65084f51be736
  ~~~

- 冷私钥

  ~~~rust
  let bob_priv_cold = r_get_my_privkey(bob_phrase, "cold").unwrap();
  ~~~

  ~~~
  44f2801002a967014760ccbf52a8aab0217934d2ecb7c38db380f5338b9d2883
  ~~~

- 热公钥

  ~~~rust
  let bob_pub_hot = r_get_my_pubkey(bob_priv_hot).unwrap();   
  ~~~

  ~~~
  03edf76a5e4b36b30218cf31ccc6081451da31f433458f60604275e346bbc22244
  ~~~

- 冷公钥

  ~~~rust
  let bob_pub_cold = r_get_my_pubkey(bob_priv_cold).unwrap(); 
  ~~~

  ~~~
  02c72ba3ca62062c921f4858418d9a79b545879f9aebd9d5abf711bc2c77e39b4d
  ~~~

Charlie

- 助记词

  ~~~rust
  let charlie_phrase = "awesome beef hill broccoli strike poem rebel unique turn circle cool system";
  ~~~

- 热私钥

  ~~~rust
  let charlie_priv_hot = r_get_my_privkey(charlie_phrase, "hot").unwrap();
  ~~~

  ~~~
  1bb59ec969f9f574f85f1cace19da8680fbb0e1f553ff9f62a0f021bbaefc8fb
  ~~~

- 冷私钥

  ~~~rust
  let charlie_priv_cold = r_get_my_privkey(charlie_phrase, "cold").unwrap();
  ~~~

  ~~~
  247c02901adf3d0b4387ae48a8d90948efd2dd4edb7286171393701f0f40fc06
  ~~~

- 热公钥

  ~~~rust
  let charlie_pub_hot = r_get_my_pubkey(charlie_priv_hot).unwrap();   
  ~~~

  ~~~
  025ba44ac870b9f6150bad39dde2b31601a88420c774dc0fb75b3cd27d82323fa4
  ~~~

- 冷公钥

  ~~~rust
  let charlie_pub_cold = r_get_my_pubkey(charlie_priv_cold).unwrap(); 
  ~~~

  ~~~
  029717b430dc7bc38356ba0a9e5fc3f1ce157659c2c829c6c66f2db909eb12b43c
  ~~~

  

  