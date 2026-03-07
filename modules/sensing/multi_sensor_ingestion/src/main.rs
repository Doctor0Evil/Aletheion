use multi_sensor_ingestion::SensorIngestionPipeline;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_line_number(true))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    tracing::info!("🏙️  Aletheion Multi-Sensor Ingestion Pipeline - Phoenix Deployment");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Initialize ingestion pipeline
    let pipeline = SensorIngestionPipeline::new().await?;
    
    // Start ingestion across all sensor types
    pipeline.start_ingestion().await?;
    
    tracing::info!("✓ Sensor ingestion pipeline operational. Press Ctrl+C to terminate.");
    
    // Keep running until shutdown signal
    tokio::signal::ctrl_c().await?;
    
    tracing::info!("Graceful shutdown initiated...");
    
    Ok(())
}
