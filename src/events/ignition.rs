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
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(8))]
pub enum Ignition {
    Off,
    Kill,
    Acc,
    Run,
    StartReceived,
    Cranking,
}

impl TryFrom<Frame> for Ignition {
    type Error = ParseError;

    /// Try to parse an [`Ignition`] event from a [`Frame`].
    fn try_from(frame: Frame) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x122;
        // the expected frame length
        const LEN: usize = 4;

        if frame.id() != ID {
            return Err(ParseError::Id { frame });
        }

        let data: [u8; LEN] = match frame.data().try_into() {
            Ok(data) => data,
            // frame not expected size
            Err(_) => {
                return Err(ParseError::Len {
                    frame,
                    expected: LEN,
                })
            }
        };

        match u32::from_be_bytes(data) {
            0x00000000 => Ok(Ignition::Off),  // off
            0x00010000 => Ok(Ignition::Off),  // off
            0x03010000 => Ok(Ignition::Kill), // engine kill
            0x03020000 => Ok(Ignition::Kill), // engine kill
            0x05020000 => Ok(Ignition::Acc),  // accessory on
            0x15020000 => Ok(Ignition::Acc),  // accessory on
            // TODO(mdegans): figure out why there are differnet values and add
            // enums for that.
            0x44010000 => Ok(Ignition::Run), // remote run (on)
            0x44020000 => Ok(Ignition::Off), // normal run (on)
            0x45010000 => Ok(Ignition::Off), // start command recv’d
            0x5d010000 => Ok(Ignition::Off), // starter is cranking
            _ => Err(ParseError::Data {
                frame,
                detail: format!(
                    "Unrecognized `Ignition` value: {:X}",
                    u32::from_be_bytes(data)
                ),
            }),
        }
    }
}
