use fancy_std::yield_now;

#[fancy_std::main]
pub async fn main() {
    loop {
        yield_now().await
    }
}
