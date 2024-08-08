
run:
    cargo run -- get http://httpbin.org/get

build:
    cargo build --release

get: build
    ./target/release/minihttpie get http://httpbin.org/get

post: build
    ./target/release/minihttpie post http://httpbin.org/post name=bob age=18
