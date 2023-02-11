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
#[derive(PartialEq, Debug, Display, Copy, Clone)]
pub struct ParkingLights(u8);
impl ParkingLights {
    /// Returns true if the parking lights are on.
    #[inline]
    pub const fn are_on(self) -> bool {
        self.0 == 1
    }
    /// Returns true if the parking lights are off.
    #[inline]
    pub const fn are_off(self) -> bool {
        !self.are_on()
    }
}

impl TryFrom<Frame<Valid>> for ParkingLights {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x2fa;
        // the expected frame length
        const LEN: usize = 8;

        if frame.id() != ID {
            return Err(ParseError::Id { frame });
        }

        let data: [u8; 8] = match frame.data().try_into() {
            Ok(data) => data,
            Err(_) => {
                return Err(ParseError::Len {
                    frame: frame.into(),
                    expected: LEN,
                })
            }
        };

        if data[1] == 0 || data[1] == 1 {
            Ok(Self(data[1]))
        } else {
            Err(ParseError::Data {
                frame,
                detail: format!(
                    "`ParkingLights` value ({}) at index 1 was neither 0 nor 1",
                    data[1]
                ),
            })
        }
    }
}

/// Interior dimmer value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Copy, Clone)]
pub struct Dimmer(u8);
impl Dimmer {
    // FIXME(mdegans): The value is always Zero on my 4xE. Maybe I am doing
    //something wrong.
    const MIN: u8 = 0;
    const MAX: u8 = 255;
    // if MIN > MAX, this will not compile
    const RANGE: u8 = Self::MAX - Self::MIN;

    #[inline]
    pub fn percent(self) -> f32 {
        f32::from(self.0 - Self::MIN) / f32::from(Self::RANGE)
    }
    #[inline]
    pub const fn is_min(self) -> bool {
        self.0 == Self::MIN
    }
    #[inline]
    pub const fn is_max(self) -> bool {
        self.0 == Self::MAX
    }
}

impl TryFrom<Frame<Valid>> for Dimmer {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x2fa;
        // the expected frame length
        const LEN: usize = 8;

        if frame.id() != ID {
            return Err(ParseError::Id { frame });
        }

        let data: [u8; 8] = match frame.data().try_into() {
            Ok(data) => data,
            Err(_) => {
                return Err(ParseError::Len {
                    frame: frame.into(),
                    expected: LEN,
                })
            }
        };

        if data[2] >= Self::MIN && data[2] <= Self::MAX {
            Ok(Self(data[2]))
        } else {
            Err(ParseError::Data {
                frame,
                detail: format!("`Dimmer` value ({}) at index 2 was outside of accepted range.", data[2]),
            })
        }
    }
}

/// A [`Lights`] related Event.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(8))]
pub enum Lights {
    /// Hazard lights
    // FIXME(mdegans): 2021 4xE Sahara - This emits when Max Regen is pressed
    HazardsOnOff,
    /// [`ParkingLights`]
    ParkingLights(ParkingLights),
    /// [`Dimmer`] state
    Dimmer(Dimmer),
}
