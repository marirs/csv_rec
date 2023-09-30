use chrono::Utc;
use clap::Parser;
use std::{error::Error, ops::Sub, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::{Mutex, RwLock, Semaphore},
};

// The point of diminishing returns for the number of concurrent tasks is the number of logical processors on the cpu,
// If set to the number of logical processors on the cpu, the program will eat up all the cpu but it will definitely be faster.
const MAX_CONCURRENT_TASKS: usize = 8;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
}

#[derive(Debug, Clone)]
enum Header {
    Good(String),
    Bad(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let start_time = chrono::Utc::now();
    let mut header = Arc::new(vec![]);
    let mut threads = vec![];
    {
        let mut readed_header = false;
        let file = File::open(&args.input).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            if !readed_header {
                header = Arc::new(
                    line.split(',')
                        .map(|s| RwLock::new(Header::Bad(s.to_string())))
                        .collect::<Vec<_>>(),
                );
                readed_header = true;
                continue;
            }
            let hh = header.clone();
            threads.push(tokio::spawn(async move {
                let ss = line.split(',');
                for (c, s) in ss.enumerate() {
                    if !s.is_empty() {
                        let sss = if let Header::Bad(ss) = &*(hh[c].read().await) {
                            Some(ss.clone())
                        } else {
                            None
                        };
                        if let Some(ssss) = sss {
                            *(hh[c].write().await) = Header::Good(ssss);
                        }
                    }
                }
            }));
        }
        println!(
            "Reading completed: {} milliseconds",
            Utc::now().sub(start_time).num_milliseconds()
        );
    }
    futures::future::join_all(threads).await;
    println!(
        "Searching empty columns completed: {} milliseconds",
        Utc::now().sub(start_time).num_milliseconds()
    );

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
    let mut threads = vec![];
    {
        let file = File::open(&args.input).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let outfile = Arc::new(Mutex::new(File::create(&args.output).await?));
        let mut header_out = false;
        while let Some(line) = lines.next_line().await? {
            if !header_out {
                let ss = line.split(',');
                let mut res = String::with_capacity(2048 * 2048);
                let mut first_out = false;
                for (c, s) in ss.enumerate() {
                    if let Header::Good(_) = &*(header[c].read().await) {
                        if !first_out {
                            res += s;
                            first_out = true;
                        } else {
                            res += ",";
                            res += s;
                        }
                    }
                }
                res += "\n";
                outfile
                    .lock()
                    .await
                    .write_all(res.as_bytes())
                    .await
                    .unwrap();
                header_out = true;
                continue;
            }
            let hh = header.clone();
            let outfile = outfile.clone();
            let semaphore = semaphore.clone();
            threads.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let ss = line.split(',');
                let mut res = String::with_capacity(2048 * 2048);
                let mut first_out = false;
                for (c, s) in ss.enumerate() {
                    if let Header::Good(_) = &*(hh[c].read().await) {
                        if !first_out {
                            res += s;
                            first_out = true;
                        } else {
                            res += ",";
                            res += s;
                        }
                    }
                }
                res += "\n";
                outfile
                    .lock()
                    .await
                    .write_all(res.as_bytes())
                    .await
                    .unwrap();
                drop(_permit);
            }));
        }
        println!(
            "Writing starting: {} milliseconds",
            Utc::now().sub(start_time).num_milliseconds()
        );
        futures::future::join_all(threads).await;
        println!(
            "Create new file completed: {} milliseconds",
            Utc::now().sub(start_time).num_milliseconds()
        );
    }
    Ok(())
}
