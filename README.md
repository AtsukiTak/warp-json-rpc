## Version 0.1

このライブラリはまだ実験段階。
特にデザインに関して、大きく変更したい。
しかしそのためには、warpのアップデートや作者の知識を増やすことが必要。
とりあえず動くものとしてversion 0.1をリリースする。

### Desired Design

```rust
use warp_json_rpc::filters as json_rpc;
use futures::future;
use warp::Filter as _;

let add_method = json_rpc::method("add")
  .and(json_rpc::params::<(u8, u8)>())
  .and_then(|a, b| future::ok(a + b));

let greet_method = json_rpc::method("greet")
  .and(json_rpc::params::<(String)>())
  .and_then(|name| future::ok(format!("Hello {}", name)))

let filter = filters::path::::path("rpc")
  .and(add_method.or(greet_method));
```

現在はこれができない。なぜなら、最初にBodyをパースした段階でBodyがextractされてしまい、次回以降のパース時に失敗するから。
これを実現するためには `Request` に対してメタデータとしてパースしたBodyを付与すればいい。
ただし、現在のwarpのデザインからは `ext::set` が削除されている。
https://github.com/seanmonstar/warp/issues/222

代替策として hyper::Service に変換するというのが提案されているが、一度Serviceに変換すると以降warpの世界に戻って来れなそう。(この辺は俺の知識不足もあるかも)
