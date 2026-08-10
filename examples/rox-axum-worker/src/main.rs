use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("ROX + MinaCalc worker listening on http://127.0.0.1:3000");
    axum::serve(listener, rox_minacalc_worker::app()).await?;
    Ok(())
}
