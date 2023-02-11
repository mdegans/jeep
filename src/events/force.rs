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

use super::{Display, Frame, Front, FrontOrRear, ParseError, Rear};

/// Road feedback from axle sensors.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Clone)]
pub struct RoadFeedback([u8; 8]);
impl TryFrom<Frame<Valid>> for FrontOrRear<RoadFeedback> {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected frame length
        const LEN: usize = 8;

        let data: [u8; LEN] = match frame.data().try_into() {
            Ok(data) => data,
            Err(_) => {
                return Err(ParseError::Len {
                    frame: frame.into(),
                    expected: LEN,
                })
            }
        };

        match frame.id() {
            0x24e => Ok(Front(RoadFeedback(data))),
            0x252 => Ok(Rear(RoadFeedback(data))),
            _ => Err(ParseError::Id { frame }),
        }
    }
}
impl std::fmt::Display for RoadFeedback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME(mdegans): decode and print valid accelerometer values.
        f.write_fmt(format_args!("RoadFeedback({:#x?})", self.0))
    }
}

/// A [`Force`] related event
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(4))]
pub enum Force {
    /// [`RoadFeedback`] from the [`FrontOrRear`] axle sensors.
    RoadFeedback(FrontOrRear<RoadFeedback>),
}

impl TryFrom<Frame<Valid>> for Force {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        match frame.id() {
            0x24e | 0x252 => Ok(Force::RoadFeedback(frame.try_into()?)),
            _ => Err(ParseError::Id { frame }),
        }
    }
}
