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
use crate::frame::Frame;

/// A [`Camera`] related event. This is guaranteed to have the same
/// representation as the byte at index 1 of a frame from id `0x302`.
#[derive(PartialEq, Debug, Display, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(align(8))]
pub enum Camera {
    Off = 0x00,
    Initializing = 0x02,
    Reverse = 0x07,
    Cargo = 0x09,
}

impl TryFrom<Frame> for Camera {
    type Error = ParseError;

    /// Try to convert from a [`Frame`] from id `0x0302` into a [`Camera`]
    /// event.
    ///
    /// # Panics
    /// In debug configurations if `frame.id() != 0x302`, since this indicates
    /// a programmer error, likely in `Event`.
    fn try_from(frame: Frame) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x302;
        // the expected frame length
        const LEN: usize = 8;

        if frame.id() != ID {
            return Err(ParseError::Id { frame });
        }

        let data: [u8; LEN] = match frame.data().try_into() {
            Ok(data) => data,
            Err(_) => {
                return Err(ParseError::Len {
                    frame,
                    expected: LEN,
                })
            }
        };

        match data[0] {
            0x00 => Ok(Camera::Off),
            0x02 => Ok(Camera::Reverse),
            0x07 => Ok(Camera::Cargo),
            0x09 => Ok(Camera::Initializing),
            _ => Err(ParseError::Data {
                frame: frame,
                detail: format!(
                    "Unrecognize {} byte at index 0: {}",
                    stringify!(Camera),
                    data[0]
                ),
            }),
        }
    }
}
