use async_channel;
use hyper::body::HttpBody;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};

const FOLDER: &str = "/mnt/raid0";
const NUM_WRITE_TASKS: usize = 16;

struct DataChunk {
    offset: u64,
    data: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:5201".parse().unwrap();
    println!("Server listening on {}", addr);

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(move |req| handle_request(req)))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if req.method() != Method::POST {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::from("Only POST requests are allowed"))
            .unwrap());
    }

    let file_name = if let Some(query) = req.uri().query() {
        let params: Vec<&str> = query.split('&').collect();
        let mut file_name = None;
        for param in params {
            let kv: Vec<&str> = param.split('=').collect();
            if kv.len() == 2 && kv[0] == "file_name" {
                file_name = Some(kv[1].to_string());
                break;
            }
        }
        if let Some(name) = file_name {
            name
        } else {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Missing file_name parameter"))
                .unwrap());
        }
    } else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Missing file_name parameter"))
            .unwrap());
    };

    println!("Receiving file: {}", file_name);

    let file_path = Path::new(FOLDER).join(&file_name);

    let (tx, rx) = async_channel::bounded::<DataChunk>(1024);

    for _ in 0..NUM_WRITE_TASKS {
        let rx = rx.clone();
        let file_path = file_path.clone();

        tokio::spawn(async move {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&file_path)
                .await
                .expect("Failed to open file");

            while let Ok(data_chunk) = rx.recv().await {
                // Seek to the offset
                if let Err(e) = file.seek(SeekFrom::Start(data_chunk.offset)).await {
                    eprintln!("Failed to seek in file: {}", e);
                    continue;
                }

                if let Err(e) = file.write_all(&data_chunk.data).await {
                    eprintln!("Failed to write to file: {}", e);
                }
            }
        });
    }

    let mut body = req.into_body();
    let mut offset = 0u64;

    while let Some(chunk_result) = body.data().await {
        match chunk_result {
            Ok(chunk) => {
                let data_len = chunk.len();
                let data_chunk = DataChunk {
                    offset,
                    data: chunk.to_vec(),
                };
                offset += data_len as u64;

                if tx.send(data_chunk).await.is_err() {
                    eprintln!("Failed to send data to writer task.");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error while reading body: {}", e);
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("Invalid request body"))
                    .unwrap());
            }
        }
    }

    drop(tx);

    println!(
        "File {} received successfully and saved to {:?}",
        file_name, file_path
    );

    Ok(Response::new(Body::from("File uploaded successfully")))
}
