mod firehose;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    firehose::listen().await?;

    Ok(())
}
