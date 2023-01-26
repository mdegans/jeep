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

/// Source of a [`Remote`] event (app, keyfob).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(4))]
pub enum RemoteSource {
    /// From UConnect, if the alignment of the planets suits and requisite blood
    /// sacrifices to Satan and/or SiriusXM Guardian have been performed.
    App,
    /// Remote event from the KeyFob. This one is nearly instant.
    KeyFob,
}

/// A [`Remote`] control event, including a secret [`Remote::DoubleUnlock`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(8))]
pub enum Remote {
    /// Idle event (this can be ignored, but is one indication the vehicle is
    /// awake).
    Idle,
    /// Remote lock request from a [`RemoteSource`].
    // TEST_PASS(mdegans): 2021 4xE Sahara (with KeyFob only)
    // TODO(mdegans): Test with App
    LockFrom(RemoteSource),
    /// Remote unlock request from a [`RemoteSource`].
    // TEST_PASS(mdegans): 2021 4xE Sahara (with KeyFob only)
    // TODO(mdegans): Test with App
    UnlockFrom(RemoteSource),
    /// Two sequential presses of the Unlock on the [`RemoteSource::KeyFob`]
    /// only.
    // TEST_PASS(mdegans): 2021 4xE Sahara
    DoubleUnlock,
    /// A Keyless entry into the Jeep, if that's enabled.
    ///
    /// # Warning
    ///
    /// It's recommended **not** to enable this feature on your Jeep because it
    /// presents a security vulnerability where a signal from your KeyFob can be
    /// extended and used to enter and start your vehicle from an distance.
    // UNTESTED(mdegans): 2021 4xE Sahara - Will Not Test
    KeylessEntry,
    /// the Jeep has been asked to start up.
    // TODO(mdegans): Test with App and KeyFob
    StartFrom(RemoteSource),
    /// Startup has been cancelled.
    // TODO(mdegans): Test with App and KeyFob
    CancelStart,
    /// Panic event (do the light show, scary announcement, distraction,
    /// whatever).
    // TEST_PASS(mdegans): 2021 4xE Sahara (with KeyFob only)
    // TODO(mdegans): Test with App
    PanicFrom(RemoteSource),
}

impl TryFrom<Frame> for Remote {
    type Error = ParseError;

    /// Try to convert a [`Frame`] into a [`Remote`] event.
    fn try_from(frame: Frame) -> Result<Self, Self::Error> {
        use RemoteSource::{App, KeyFob};

        // the expected `frame.id` for this event.
        const ID: u32 = 0x1c0;
        // the expected frame length
        const LEN: usize = 6;

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

        // the first byte is enough to match any remote event
        match data[0] {
            0x00 => Ok(Remote::Idle),               // none(idle)
            0x21 => Ok(Remote::LockFrom(KeyFob)),   // - keyfob lock
            0x23 => Ok(Remote::UnlockFrom(KeyFob)), // - 1st press fob unlock
            0x24 => Ok(Remote::DoubleUnlock),       // - 2nd press fob unlock
            0x2E => Ok(Remote::PanicFrom(KeyFob)),  // - keyfob panic button
            0x43 => Ok(Remote::KeylessEntry),       // - driver keyless entry
            0x69 => Ok(Remote::StartFrom(KeyFob)),  // - keyfob remote start
            0x81 => Ok(Remote::LockFrom(App)),      // - app lock doors
            0x83 => Ok(Remote::UnlockFrom(App)),    // - app unlock doors
            0x6A => Ok(Remote::CancelStart),        // - (any) cancel rem start
            // rust says this code is unreachable (because dupe). Guessing
            // one of these is supposed to be 0x82
            // 0x83 => Ok(Remote::PanicFrom(App)),  // – app panic button
            _ => Err(ParseError::Data {
                frame: frame,
                detail: format!(
                    "Byte at index 0 not recognized: {:X}",
                    data[0]
                ),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_parsing() {
        let frame =
            Frame::from_id_data_len(0x1c0, [0x21, 0, 0, 0, 0, 0, 0, 0], 6)
                .unwrap();
        let remote = Remote::try_from(frame).unwrap();

        assert_eq!(remote, Remote::LockFrom(RemoteSource::KeyFob))
    }
}
