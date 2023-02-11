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

use crate::frame::state::Valid;

use super::{Display, Frame, ParseError};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
pub struct Temperature(u16);
impl Temperature {
    pub fn in_celsius(self) -> f32 {
        let value = f32::from(self.0);

        (value / 100.0) - 40.0
    }
    pub fn in_farenheit(self) -> f32 {
        (self.in_celsius() * (9.0 / 5.0)) + 32.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(8))]
pub enum HVAC {
    Cabin(Temperature),
}

impl TryFrom<Frame<Valid>> for HVAC {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x33a;
        // the expected frame length
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

        Ok(HVAC::Cabin(Temperature(u16::from_be_bytes([
            data[0], data[1],
        ]))))
    }
}
