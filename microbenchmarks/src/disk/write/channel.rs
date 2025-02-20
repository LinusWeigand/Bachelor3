use clap::Parser;
use rand::{rng, Rng};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Mutex;

const FILE_PATH: &str = "/mnt/raid0/testfile";

const CHANNEL_BUFFER_SIZE: usize = 1024 * 1024 * 256;

struct DataChunk {
    offset: u64,
    data: Vec<u8>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "7")]
    write: u64,
    #[arg(short, long, default_value = "2")]
    send: u64,
    #[arg(short, long, default_value = "256")]
    size: u64,
    #[arg(short, long, default_value = "50")]
    duration: u64,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let duration = args.duration;
    let block_size = args.size * 1024 * 1024;
    let total_bytes_written = Arc::new(Mutex::new(0u64));
    let (tx, rx) = async_channel::bounded::<Arc<DataChunk>>(CHANNEL_BUFFER_SIZE);

    let write_tasks: Vec<_> = (0..args.write)
        .map(|i| {
            let total_bytes_written = Arc::clone(&total_bytes_written);
            let rx = rx.clone();

            tokio::spawn(async move {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(FILE_PATH)
                    .await
                    .expect("Failed to open file");

                let start_time = Instant::now();
                let mut bytes_written = 0u64;

                while start_time.elapsed().as_secs() < duration {
                    if let Ok(data_chunk) = rx.recv().await {
                        file.seek(SeekFrom::Start(data_chunk.offset))
                            .await
                            .expect("Seek failed");
                        file.write_all(&data_chunk.data)
                            .await
                            .expect("Write failed");

                        bytes_written += data_chunk.data.len() as u64;
                    }
                }

                let mut total = total_bytes_written.lock().await;
                *total += bytes_written as u64;
                println!(
                    "Task {}: Bytes: {}, MiB/s: {}",
                    i,
                    bytes_written,
                    (bytes_written / (1024 * 1024) / start_time.elapsed().as_secs())
                );
            })
        })
        .collect();

    let send_tasks: Vec<_> = (0..args.send)
        .map(|i| {
            let tx = tx.clone();

            tokio::spawn(async move {
                let mut offset = i as u64 * block_size;

                let start_time = Instant::now();
                while start_time.elapsed().as_secs() < duration {
                    let data = create_random_buffer(CHANNEL_BUFFER_SIZE);

                    let data_chunk = Arc::new(DataChunk { offset, data });

                    if tx.send(Arc::clone(&data_chunk)).await.is_err() {
                        eprintln!("Failed to put data_chunk into channel");
                        return;
                    }
                    offset += args.send * block_size;
                }
            })
        })
        .collect();

    futures::future::join_all(send_tasks).await;

    drop(tx);
    futures::future::join_all(write_tasks).await;

    let elapsed = duration as f64;
    let total_bytes = *total_bytes_written.lock().await;
    let total_throughput_mib = (total_bytes as f64 / 1_048_576.0) / elapsed;
    println!(
        "Total: Written {} bytes, Throughput: {:.2} MiB/s",
        total_bytes, total_throughput_mib
    );

    Ok(())
}

fn create_random_buffer(size: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; size];
    rng().fill(&mut buffer[..]);
    buffer
}
