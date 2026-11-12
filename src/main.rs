use clap::{Parser, ValueEnum};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::{
    format,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Semaphore, task::JoinHandle};

#[derive(Clone, ValueEnum, Debug)]
enum Mode {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

// Arguements
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    // Main configurations
    #[arg(
        long,
        help = "URL to send the requests to, including the scheme (for example, https://example.com)"
    )]
    url: String,

    #[arg(long, value_enum, help = "HTTP method to use for every request")]
    method: Mode,

    #[arg(
        long,
        value_parser = clap::value_parser!(i32).range(1..),
        help = "Total number of requests to send"
    )]
    request_count: i32,

    #[arg(
        long,
        value_parser = clap::value_parser!(i32).range(1..),
        help = "Maximum number of requests to run at the same time"
    )]
    concurrency: i32,

    // Optional configurations
    #[arg(
        long,
        default_value = "{}",
        help = "Request body to send with each request; defaults to no body"
    )]
    body: Option<String>,

    #[arg(
        long,
        short='H',
        value_name = "NAME:VALUE",
        action = clap::ArgAction::Append,
        help = "HTTP header to include; repeat this option to add multiple headers"
    )]
    headers: Vec<String>,

    #[arg(
        long,
        short,
        help = "Print each request's success status, response body, and elapsed time"
    )]
    debug: bool,
}

// Request result structure
struct RequestResult {
    success: bool,
    elapsed: Duration,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let url = cli.url;
    let method = cli.method;
    let body = cli.body.unwrap();
    let headers = cli.headers;
    let request_count = cli.request_count;
    let concurrency = cli.concurrency;
    let debug = cli.debug;

    println!("************** Arguments **************");
    println!("- URL: {:?}", url);
    println!("- METHOD: {:?}", method);
    println!("- BODY: {:?}", body);
    println!("- HEADERS: {:?}", headers);
    println!("- COUNT: {:?}", request_count);
    println!("- CONCURRENCY: {:?}", concurrency);
    println!("- DEBUG: {:?}", debug);

    let handles = create_tasks(
        concurrency as usize,
        request_count as usize,
        url,
        method,
        headers,
        body,
    )
    .await?;

    let results = execute_tasks(request_count as usize, handles, debug).await;

    let successful = results.iter().filter(|r| r.success).count();
    let total_elapsed: Duration = results.iter().map(|r| r.elapsed).sum();

    let average = if results.is_empty() {
        0.0
    } else {
        total_elapsed.as_secs_f64() / results.len() as f64
    };

    println!("************** Results **************");
    println!("- SUCCESSFUL: {successful}/{request_count}");
    println!("- AVERAGE RESPONSE TIME: {:.3} seconds", average);
    Ok(())
}

async fn create_tasks(
    concurrency: usize,
    count: usize,
    url: String,
    method: Mode,
    headers: Vec<String>,
    body: String,
) -> Result<Vec<JoinHandle<RequestResult>>, Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let mut handles = Vec::with_capacity(count);

    for _ in 0..count {
        let permit = semaphore.clone().acquire_owned().await?;
        let client = client.clone();
        let url = url.clone();
        let method = method.clone();
        let body = body.clone();

        let mut request = match method {
            Mode::GET => client.get(&url),
            Mode::POST => client.post(&url),
            Mode::PUT => client.put(&url),
            Mode::PATCH => client.patch(&url),
            Mode::DELETE => client.delete(&url),
        };

        if !&headers.is_empty() {
            request = request.headers(parse_headers(&headers)?);
        }

        if &body != "{}" {
            request = request.body(body.clone());
        }

        handles.push(tokio::spawn(async move {
            let started = Instant::now();

            let response = request.send().await;

            let elapsed = started.elapsed();
            let success = matches!(&response, Ok(response) if response.status().is_success());
            let message = match response {
                Ok(response) => response
                    .text()
                    .await
                    .unwrap_or_else(|err| format!("Error reading body: {err}")),
                Err(err) => format!("Request error: {err}"),
            };

            // Dropping `permit` here releases its concurrency slot.
            drop(permit);

            RequestResult {
                success,
                elapsed,
                message,
            }
        }));
    }

    Ok(handles)
}

fn parse_headers(raw_headers: &[String]) -> Result<HeaderMap, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();

    for raw in raw_headers {
        let (name, value) = raw
            .split_once(':')
            .ok_or_else(|| format!("invalid header, expected NAME:VALUE: {raw}"))?;

        let name = HeaderName::from_bytes(name.trim().as_bytes())?;
        let value = HeaderValue::from_str(value.trim())?;
        headers.insert(name, value);
    }

    Ok(headers)
}

async fn execute_tasks(
    count: usize,
    handles: Vec<JoinHandle<RequestResult>>,
    debug: bool,
) -> Vec<RequestResult> {
    let mut results = Vec::with_capacity(count as usize);
    for handle in handles {
        let result = handle.await.expect("request task panicked");
        let debug = debug.clone();

        if debug {
            // Message display
            let elapsed = &result.elapsed.as_secs_f64().to_string();
            let success = &result.success;
            let message = &result.message;
            println!("Request processed! Success: {success} Message: {message:?} Time: {elapsed}");
        }

        results.push(result);
    }

    results
}
