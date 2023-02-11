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

bitflags::bitflags! {
    /// [`bitflags`] representing the state of the jeep's steering wheel
    /// buttons with the exception of cruise control. More than one button can
    /// be pressed at a time, enabling button combinations to do special things
    /// without having to remove one's hands from the wheel. All 16 bits of
    /// bytes 3 and 4 in a frame from 0x318 can be used, provided you have such
    /// a steering wheel.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[repr(align(8))]
    #[derive(Display)]
    pub struct Buttons: u16 {
        // front buttons
        /// (Left) d-pad left arrow.
        const DPAD_LEFT                 = 0b00000000_00000001;
        /// (Unidentifed) custom steering wheel button 0.
        const MYSTERY_BTN_0             = 0b00000000_00000010;
        /// (Left) d-pad down arrow.
        const DPAD_DOWN                 = 0b00000000_00000100;
        /// (Unidentifed) custom steering wheel button 1.
        const MYSTERY_BTN_1             = 0b00000000_00001000;
        /// (Left) d-pad up arrow.
        const DPAD_UP                   = 0b00000000_00010000;
        /// (Unidentifed) custom steering wheel button 2.
        const MYSTERY_BTN_2             = 0b00000000_00100000;
        /// (Left) d-pad right arrow.
        const DPAD_RIGHT                = 0b00000000_01000000;
        /// (Unidentifed) custom steering wheel button 3.
        const MYSTERY_BTN_3             = 0b00000000_10000000;

        // rear buttons
        /// (Rear) input select button.
        const BACK_INPUT_BUTTON         = 0b00000001_00000000;
        /// (Unidentifed) custom steering wheel button 4.
        const MYSTERY_BTN_4             = 0b00000010_00000000;
        /// (Rear) volume up button.
        const BACK_VOL_UP               = 0b00000100_00000000;
        /// (Rear) volume down button.
        const BACK_VOL_DOWN             = 0b00001000_00000000;
        /// (Rear) track skip button.
        const BACK_TRACK_SKIP           = 0b00010000_00000000;
        /// (Rear) previous track button.
        const BACK_TRACK_REWIND         = 0b00100000_00000000;
        /// (Rear) seek button.
        const BACK_SEEK_BUTTON          = 0b01000000_00000000;
        /// (Unidentifed) custom steering wheel button 5.
        const MYSTERY_BTN_5             = 0b10000000_00000000;

        // masks
        /// Msk for stock buttons on the Jeep Wrangler. Only ones exist.
        /// (no `MYSTERY_BTN_?`)
        const STOCK_BUTTONS             = 0b01111101_01010101;
    }
}

impl Buttons {
    /// The stock steering wheel buttons (except cruise control) on the Jeep
    /// Wrangler. If you explicitly do not want to support custom steering wheel
    /// presses, use this and "MYSTERY_BTN" bits will be masked out.
    pub const fn stock_buttons_pressed(self) -> Self {
        self.intersection(Self::STOCK_BUTTONS)
    }
}

impl TryFrom<Frame<Valid>> for Buttons {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x318;
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

        match Buttons::from_bits(u16::from_be_bytes([data[3], data[4]])) {
            Some(dpad) => Ok(dpad),
            // There are bits that do not correspond to a flag. This should
            // never happen with `steering_wheel::Buttons`
            None => Err(ParseError::Data {
                frame,
                detail: "There were bits that do not correspond to a flag. This means the `steering_wheel::Buttons` code is broken since every bit should have a flag.".to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Buttons, Frame};

    #[test]
    fn stock_buttons_pressed() {
        let all_buttons_pressed = Frame::from_id_data_len(
            0x318,
            [0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00],
            8,
        )
        .unwrap();
        let parsed = Buttons::try_from(all_buttons_pressed).unwrap();
        assert_eq!(Buttons::all(), parsed);
        assert_eq!(parsed.stock_buttons_pressed(), Buttons::STOCK_BUTTONS);
    }
}
