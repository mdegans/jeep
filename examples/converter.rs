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

use jeep::Event;

use clap::Parser;
use serde::{Deserialize, Serialize};
use socketcan::CANFrame;

use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Parse a `candump -L` file into events or errors",
    long_about = None
)]
struct Args {
    /// Candump file
    #[arg(short, long)]
    in_file: String,

    /// Json lines output file.
    #[arg(short, long)]
    out_file: String,

    /// IDs to filter by
    #[arg(short, long, value_parser=clap_num::maybe_hex::<u32>)]
    filters: Option<Vec<u32>>,
}

/// parse a candump (-L) line into (timestamp, interface, id, data)
fn parse_candump_line(
    line: &str,
    filters: &Option<Vec<u32>>,
) -> Option<(u128, CANFrame)> {
    // FIXME? use regex instead?
    let split: Vec<&str> = line.split(['.', ' ', '#']).collect();
    let components: [&str; 5] = split.try_into().ok()?;
    let [timestamp_sec, timestamp_subsec, _, id, hex_data] = components;

    // check id first, so we can quickly filter
    let id = u32::from_str_radix(id, 16).ok()?;
    if let Some(filters) = filters {
        if !filters.contains(&id) {
            return None;
        }
    }

    let timestamp_sec = timestamp_sec.replace('(', "");
    let timestamp_subsec = timestamp_subsec.replace(')', "");
    let timestamp: u128 = (timestamp_sec.to_owned() + &timestamp_subsec)
        .parse()
        .ok()?;

    // https://stackoverflow.com/questions/52987181/how-can-i-convert-a-hex-string-to-a-u8-slice
    let data: Vec<u8> = (0..hex_data.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex_data[i..i + 2], 16)
                .expect(&format!("invalid hex character in data:{line}"))
        })
        .collect();

    debug_assert!(data.len() <= 8);

    Some((timestamp, CANFrame::new(id, &data, false, false).ok()?))
}

fn write_json<W, M>(
    writer: &mut W,
    message: &M,
    timestamp: u128,
) -> Result<(), Box<dyn std::error::Error>>
where
    M: Serialize,
    W: std::io::Write,
{
    #[derive(Serialize, Deserialize)]
    struct TimestampedPayload<P> {
        timestamp: u128,
        payload: P,
    }

    let json = serde_json::to_string(&TimestampedPayload {
        timestamp,
        payload: message,
    })?;

    writeln!(writer, "{json}")?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let in_file = File::open(args.in_file)?;
    let mut out_file = File::create(args.out_file)?;
    let mut lines = BufReader::new(in_file).lines();

    while let Some(Ok(line)) = lines.next() {
        if let Some((timestamp, frame)) =
            parse_candump_line(&line, &args.filters)
        {
            let result = Event::parse(frame);
            write_json(&mut out_file, &result, timestamp)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_candump_line, CANFrame};

    #[test]
    fn test_from_candump_line() {
        let filters = Some(vec![0x44, 0x236]);
        let lines_frames: [(&str, Option<CANFrame>); 3] = [
            (
                "(1436509052.249713) vcan0 044#2A366C2BBA",
                Some(
                    CANFrame::new(
                        0x044,
                        &[0x2A, 0x36, 0x6C, 0x2B, 0xBA],
                        false,
                        false,
                    )
                    .unwrap(),
                ),
            ),
            ("(1436509052.449847) vcan0 0F6#7ADFE07BD2", None),
            (
                "(1436509052.650004) vcan0 236#C3406B09F4C88036",
                Some(
                    CANFrame::new(
                        0x236,
                        &[0xC3, 0x40, 0x6B, 0x09, 0xF4, 0xC8, 0x80, 0x36],
                        false,
                        false,
                    )
                    .unwrap(),
                ),
            ),
        ];
        for (line, expected) in lines_frames {
            let actual = parse_candump_line(line, &filters);
            if let Some((_, frame)) = actual {
                assert_eq!(frame.id(), expected.unwrap().id());
                assert_eq!(frame.data(), expected.unwrap().data());
            } else {
                assert!(expected.is_none());
            }
        }
    }
}
