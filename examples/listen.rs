// MIT License

// Copyright (c) 2023 Michael de Gans

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use clap::Parser;
use jeep::listener::{Error, Listener, Message};
use serde::{Deserialize, Serialize};

use std::{
    fs::File,
    sync::mpsc::sync_channel,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

// Polling rate for batches of messages (default 1/120 sec).
const PAUSE: std::time::Duration = std::time::Duration::from_nanos(8330000);

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum DumpFormat {
    /// Json lines, including timestamp.
    Json,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Listen to Jeep events on the CAN bus",
    long_about = None
)]
struct Args {
    /// CAN interface to open (eg. "can0").
    #[arg(long)]
    device: String,
    /// Print ERRORs as well as OK results.
    #[arg(short, long)]
    verbose: bool,
    /// Dump frames to this file as json lines.
    #[arg(long)]
    dump: Option<String>,
}

fn ns_since_unix_epoch() -> Result<u128, Box<dyn std::error::Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())
}

fn write_json<W>(
    writer: &mut W,
    message: &Message,
    timestamp: u128,
) -> Result<(), Box<dyn std::error::Error>>
where
    W: std::io::Write,
{
    #[derive(Serialize, Deserialize)]
    struct TimestampedPayload<P> {
        timestamp: u128,
        payload: P,
    }

    let json = match message {
        Ok(event) => serde_json::to_string(&TimestampedPayload {
            timestamp,
            payload: event,
        })?,
        Err(err) => match err {
            Error::ParseError(err) => {
                serde_json::to_string(&TimestampedPayload {
                    timestamp,
                    payload: err,
                })?
            }
            Error::IoError(err) => {
                serde_json::to_string(&TimestampedPayload {
                    timestamp,
                    // std::io::Error has no Serialize, so we just write it as
                    // string in the payload for now.
                    payload: format!("{err}"),
                })?
            }
            // we migth want to panic here?
            Error::BadLen(err) => serde_json::to_string(&TimestampedPayload {
                timestamp,
                payload: err,
            })?,
        },
    };

    writeln!(writer, "{json}")?;

    Ok(())
}

/// Print a [`Message`] (an [`jeep::Event`] or [`Error`])
fn print_message(timestamp: u128, message: &Message, verbose: bool) {
    match message {
        Ok(event) => println!("{timestamp}:Ok({event})"),
        Err(error) => {
            if verbose {
                println!("{timestamp}:Err({error})")
            }
        }
    }
}

fn handle_message<W>(
    writer: &mut W,
    message: Message,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>>
where
    W: std::io::Write,
{
    let timestamp = ns_since_unix_epoch()?;

    print_message(timestamp, &message, verbose);
    write_json(writer, &message, timestamp)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // open dump file
    let mut dump = match args.dump {
        Some(filename) => Some(File::create(filename)?),
        None => None,
    };
    // listener in non-blocking mode should be polled peridically for pending
    // listener.messages().
    let listener = Listener::connect(&args.device, false)?;
    // A channel to connect the ctrl+c signal handler thread to the main loop.
    let (tx, rx) = sync_channel(0);
    ctrlc::set_handler(move || tx.send(()).expect("rx disconnected somehow."))
        .expect("Error setting Ctrl-C handler");

    // main event loop
    loop {
        // to prevent spinning at the end of the loop
        let loop_start = Instant::now();

        // parse all pending messages
        for message in listener.messages() {
            if let Some(file) = &mut dump {
                handle_message(file, message, args.verbose)?;
            }
        }

        // Check ctrl+c receiver.
        if rx.try_recv().is_ok() {
            println!("CTRL+C Received.");
            // break out of the main loop
            break;
        }

        // Pause unless somehow parsing messages took more than 1/120 sec.
        // This is not guaranteed to result in a 120hz poll, but it's close
        // enough to throttle so we don't spin quite so hard.
        let elapsed = loop_start.elapsed();
        if elapsed < PAUSE {
            std::thread::sleep(PAUSE - elapsed);
        }
    }

    Ok(())
}
