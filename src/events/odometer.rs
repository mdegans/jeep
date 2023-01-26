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

use super::{Display, Frame, ParseError};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Copy, Clone)]
#[repr(align(8))]
pub struct Odometer(u32);
impl Odometer {
    /// value as kilometers, down to the 100th kilometer.
    pub fn kilometers(self) -> f64 {
        f64::from(self.0) / 100.0
    }
    /// value as miles, down to the 100th kilometer.
    pub fn miles(self) -> f64 {
        self.kilometers() * 0.621371
    }
    /// raw odometer bits as 100ths of a kilometer
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl TryFrom<Frame> for Odometer {
    type Error = ParseError;

    fn try_from(frame: Frame) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x3d2;
        // the expected frame length
        const LEN: usize = 4;

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

        Ok(Odometer(u32::from_be_bytes(data)))
    }
}
