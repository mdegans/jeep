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

/// Cause of a [`Bus::Wake`]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
pub enum Wake {
    HoodOpen,
    HoodClose,
    Unplug,
    Plug,
}

/// A Bus status event.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(8))]
pub enum Bus {
    /// A bus wake event. (Usually) the first thing sent on the bus.
    Wake(Wake),
}

impl TryFrom<Frame<Valid>> for Bus {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        const ID: u32 = 0x401;
        const LEN: usize = 8;

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

        // FIXME(mdegans): this is almost assuredly wrong
        match data[4..6] {
            [0x01, 0x03] => Ok(Bus::Wake(Wake::Plug)),
            [0x01, 0x04] => Ok(Bus::Wake(Wake::Unplug)),
            [0x0c, 0x06] => Ok(Bus::Wake(Wake::HoodOpen)),
            [0x0c, 0x07] => Ok(Bus::Wake(Wake::HoodClose)),
            _ => Err(ParseError::Data {
                detail: format!(
                    "Unrecognized {} data in frame: {}",
                    stringify!(Bus),
                    &frame
                ),
                frame,
            }),
        }
    }
}
