#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = tor_socks_gui_lib::cli::run(&args).await;
    std::process::exit(code);
}
