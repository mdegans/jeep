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

use super::{Display, ParseError};
use crate::frame::{state::Valid, Frame};

/// The 12v (starter) battery under the hood that powers the "Aux" stuff.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Aux([u8; 4]);
impl Aux {
    /// The raw data from the frame, the first two bytes of which are
    /// unidentified. Notes in spreadsheet say "Unknown. Charge? Load?".
    /// If you can figure out what they do, please write accessor methods
    /// for the Aux impl (using `const fn` if possible), and submit a PR.
    pub const fn raw(self) -> [u8; 4] {
        self.0
    }
    /// The byte containing the raw 1/10 volts as u8.
    pub const fn raw_volts(self) -> u8 {
        self.0[2]
    }
    /// The voltage of the Aux battery.
    pub fn volts(self) -> f32 {
        f32::from(self.raw_volts()) / 100.0
    }
}

impl std::fmt::Display for Aux {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}({:.2})", stringify!(Self), self.volts()))
    }
}

impl TryFrom<Frame<Valid>> for Aux {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x2c2;
        // the expected frame length
        const LEN: usize = 4;

        if frame.id() != ID {
            return Err(ParseError::Id { frame });
        }

        let data: [u8; LEN] = match frame.data().try_into() {
            Ok(data) => data,
            Err(_) => {
                return Err(ParseError::Len {
                    frame: frame.into(),
                    expected: LEN,
                })
            }
        };

        Ok(Aux(data))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(8))]
pub enum Battery {
    Aux(Aux),
}

impl TryFrom<Frame<Valid>> for Battery {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        match frame.id() {
            0x2c2 => Ok(Battery::Aux(frame.try_into()?)),
            // 4xE big battery goes here
            _ => Err(ParseError::Id { frame }),
        }
    }
}
