Attempt to MVCE for https://github.com/rust-lang/rust/issues/70314

Steps to reproduce the bug
```
git clone https://github.com/woshilapin/refinery
cd refinery
git checkout mvce-70314
sudo docker run -v $(pwd):/usr/local/lib/refinery -it --rm
rustlang/rust:nightly bash
cd /usr/local/lib/refinery/refinery
cargo test --features rusqlite
```
