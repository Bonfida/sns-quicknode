#[actix_web::main]
async fn main() -> std::io::Result<()> {
    sns_quicknode::main().await
}
