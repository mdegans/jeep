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

use super::{Display, Frame, From, ParseError};

/// An Event from the main [`ControlPanel`] below the head unit.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, From, Clone)]
#[repr(align(8))]
pub enum ControlPanel {
    /// [`Buttons`] pressed from source `0x2d3` with the exception of
    /// [`Warmers`] that come from another ID.
    // FIXME(mdegans): Encapsulate `Buttons` and `Warmers` in a wrapper sinc
    // they are both buttons. Right now the raw bitflags are exposed and
    // that's a less than ideal approach. Doors and locks have the same issue.
    // `bitflags` adds too many methods that might be confusing, like `all` and
    // `any` where in some cases like doors, they do not represent all doors.
    Buttons(Buttons),
    /// [`Warmers`] button press(es) from source `0x2d4`
    Warmers(Warmers),
    /// Control panel [`Knobs`] turned from source `0x273`
    Knobs(Knobs),
    // TODO(mdegans): There are buttons unaccounted for. Whether they come from
    // a different ID or are some unused bits in the above flags is unknown.
}

impl TryFrom<Frame<Valid>> for ControlPanel {
    type Error = ParseError;

    /// Try to parse a [`ControlPanel`] event from a [`Frame`].
    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        match frame.id() {
            0x2d3 => Ok(ControlPanel::Buttons(frame.try_into()?)),
            0x2d4 => Ok(ControlPanel::Warmers(frame.try_into()?)),
            0x273 => Ok(ControlPanel::Knobs(frame.try_into()?)),
            _ => Err(ParseError::Id { frame }),
        }
    }
}

bitflags::bitflags! {
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Display)]
    pub struct Buttons: u64 {
        const TRACTION_CONTROL      = 0x07000000_00000001; // Traction control on/off
        const RADIO_POWER           = 0x07000000_00000040; // Radio on/off
        const AC                    = 0x07000000_00000100; // A/C system on/off
        const RECIRCULATION         = 0x07000000_00000200; // Air recirculation on/off
        const VENT_MODE             = 0x07000000_00000800; // HVAC vent mode
        const HVAC_POWER            = 0x07000000_00010000; // HVAC sytem on/off
        const AUTO                  = 0x07000000_00020000; // Automatic HVAC control
        const DRIVER_TEMP_UP        = 0x07000000_00040000; // Driver temp +
        const DRIVER_TEMP_DOWN      = 0x07000000_00080000; // Driver temp -
        const PASSENGER_TEMP_UP     = 0x07000000_00100000; // Passenger temp +
        const PASSENGER_TEMP_DOWN   = 0x07000000_00200000; // Passenger temp -
        const REAR_DEFROSTER        = 0x07000000_00400000; // Rear defroster
        const FRONT_DEFROSTER       = 0x07000000_00800000; // Front defroster
        const MUTE                  = 0x07000100_00000000; // uConnect mute on/off
        const SCREEN                = 0x07002000_00000000; // uConnect screen on/off
        const ESS_MAX_REGEN         = 0x07240000_00000000; // ESS system on/off"
    }
}

impl TryFrom<Frame<Valid>> for Buttons {
    type Error = ParseError;

    /// Convert from a [`Frame`] to a [`Buttons`] button press.
    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x2d3;
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

        match Self::from_bits(u64::from_be_bytes(data)) {
            Some(flags) => Ok(flags),
            // unrecognized bit is set
            None => Err(ParseError::Data {
                frame,
                detail: format!("A bit was set for `{}` that doesn't correspond to a flag: {:?}", stringify!(Buttons), &data),
            }),
        }
    }
}

bitflags::bitflags! {
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Display)]
    #[repr(align(4))]
    pub struct Warmers:u16 {
        const DRIVER_BUTT    = 0x0001; // driver seat heater
        const PASSENGER_BUTT = 0x0010; // passenger seat heater
        const STEERING_WHEEL = 0x4000; // steering wheel heater
    }
}

impl TryFrom<Frame<Valid>> for Warmers {
    type Error = ParseError;

    /// Convert from a [`Frame`] to a [`Warmers`] button press.
    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x2d4;
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

        match Self::from_bits(u16::from_be_bytes([data[1], data[2]])) {
            Some(flags) => Ok(flags),
            // unrecognized bit is set
            None => Err(ParseError::Data {
                frame,
                detail: format!("A bit was set for `{}` that doesn't correspond to a flag: {:?}", stringify!(Warmers), &data),
            }),
        }
    }
}

// FIXME(mdegans): this should be bitflags
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(4))]
pub enum Knobs {
    FanDown,
    FanUp,
}

impl TryFrom<Frame<Valid>> for Knobs {
    type Error = ParseError;

    /// Convert from a [`Frame`] to a [`Knobs`] event.
    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x273;
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

        match u64::from_be_bytes(data) {
            0x00000A0000000000 => Ok(Knobs::FanDown), //fan down
            0x0000050000000000 => Ok(Knobs::FanUp),   //fan up
            0x0000090000000000 => Ok(Knobs::FanUp),   //fan up also?!
            // 0x???????????????? => Ok(Knobs::FanUp),//tune up [TBD]
            // 0x???????????????? => Ok(Knobs::FanUp),//tune down [TBD}"
            _ => Err(ParseError::Data {
                frame,
                detail: format!("Unrecognized value for `Knobs` ({:X}). Please report this.", u64::from_be_bytes(data)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hvac_radio_ess() {
        let data = Buttons::TRACTION_CONTROL
            .union(Buttons::MUTE)
            .bits
            .to_be_bytes();
        let frame = Frame::from_id_data_len(0x2d3, data, 8).unwrap();

        let parsed = Buttons::try_from(frame).unwrap();

        assert!(parsed.contains(Buttons::TRACTION_CONTROL));
        assert!(parsed.contains(Buttons::MUTE));
    }

    #[test]
    fn test_warmers() {
        let bytes = Warmers::DRIVER_BUTT.bits.to_be_bytes();
        let frame = Frame::from_id_data_len(
            0x2d4,
            [0, bytes[0], bytes[1], 0, 0, 0, 0, 0],
            8,
        )
        .unwrap();
        let parsed = Warmers::try_from(frame).unwrap();

        assert_eq!(parsed, Warmers::DRIVER_BUTT);
    }

    #[test]
    fn test_buttons() {
        let frame = Frame::from_id_data_len(
            0x2d3,
            Buttons::TRACTION_CONTROL
                .union(Buttons::MUTE)
                .bits
                .to_be_bytes(),
            8,
        )
        .unwrap();

        if let ControlPanel::Buttons(pressed) =
            ControlPanel::try_from(frame).unwrap()
        {
            assert!(pressed.contains(Buttons::TRACTION_CONTROL));
            assert!(pressed.contains(Buttons::MUTE));
        } else {
            panic!("Buttons::try_from(frame: Frame) parsed incorrect id.");
        }

        let bytes = Warmers::DRIVER_BUTT.bits.to_be_bytes();
        let frame = Frame::from_id_data_len(
            0x2d4,
            [0, bytes[0], bytes[1], 0, 0, 0, 0, 0],
            8,
        )
        .unwrap();
        if let ControlPanel::Warmers(pressed) =
            ControlPanel::try_from(frame).unwrap()
        {
            assert!(pressed.contains(Warmers::DRIVER_BUTT));
            assert!(!pressed.contains(Warmers::PASSENGER_BUTT));
        } else {
            panic!("Buttons::try_from(frame: Frame) parsed incorrect id.");
        }
    }
}
