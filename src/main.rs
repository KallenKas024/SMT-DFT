#[tokio::main]
async fn main() {
    let _g_logger: clia_tracing_config::WorkerGuard = clia_tracing_config::build()
        .directory("./dft/logs/")
        .filter_level("info")
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_source_location(true)
        .with_target(true)
        .file_name("lastrun.log")
        .rolling("daily")
        .to_stdout(true)
        .init();

    tracing::info!("Start DFT-System");
}
