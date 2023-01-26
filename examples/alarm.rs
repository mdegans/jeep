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
use jeep::{
    events::Event,
    listener::{Error, Listener},
};

use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitCode, ExitStatus},
};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Run a command when any doors are opened, then exit",
    long_about = None
)]
struct Args {
    /// Seconds to sleep before arming.
    #[arg(long, default_value_t = 0)]
    delay: u64,

    /// CAN interface to open (eg. "can0").
    #[arg(short, long)]
    device: String,

    /// Executable to run when any doors are opened.
    #[arg(short, long)]
    exe: String,

    #[arg(short, long)]
    /// Argments for the --exe to be run
    args: Vec<String>,
}

/// Helper function to convert an ExitStatus to ExitCode
fn status_to_exitcode(status: ExitStatus) -> ExitCode {
    match status.code() {
        // The exit code is never be outside of u8 range on Unix.
        Some(code) => (code as u8).into(),
        // Process was terminated by some signal (eg, through top)
        None => {
            eprintln!(
                "--exe terminated by signal: {}",
                status.signal().unwrap()
            );
            ExitCode::FAILURE
        }
    }
}

/// Print countdown for `timeout` seconds, then "!!!ARMED!!!"
fn arm_delay(timeout: u64) {
    for seconds_left in timeout..0 {
        println!("ARMING_IN:{seconds_left}");
        std::thread::sleep(std::time::Duration::from_secs(1))
    }
    println!("!!!ARMED!!!");
}

fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let args = Args::parse();
    // We use listener in blocking mode, which will block until there are more
    // messages instead of returning when messages() are exhausted.
    let listener = Listener::connect(&args.device, true)?;

    // Do any delay before arming.
    arm_delay(args.delay);

    // Listen for messages on the CAN bus (blocking)
    for message in listener.messages() {
        match message {
            // We only care about door messages
            Ok(Event::Doors(doors)) => {
                if doors.any_open() {
                    // launch the command and block until completion
                    let status =
                        Command::new(args.exe).args(args.args).status()?;

                    // convert the status code to an error code and exit
                    return Ok(status_to_exitcode(status));
                }
            }
            // IO Error, so we probably want to exit, although in the future we
            // could retry or notify seom server of the error.
            Err(Error::IoError(e)) => return Err(Box::new(e)),
            // ignore anything else (eg. ParseError, other events)
            _ => continue,
        }
    }

    // We should never actually reach here when the Listener is blocking.
    panic!("`Messages` iterator broken -- yielded no messages")
}
