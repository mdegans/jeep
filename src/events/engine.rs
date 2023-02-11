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

use super::{
    Display, Event, Frame, OneOrMany,
    OneOrMany::{Many, One},
    ParseError,
};

/// The Jeep's speed in legacy units. This can be converted to and from [`KPH`]
/// losslessly with `from` and `into`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MPH(pub(crate) u16);
impl MPH {
    /// The raw u16 value, which is 200x the actual MPH.
    pub const fn raw(self) -> u16 {
        self.0
    }
}
impl From<u8> for MPH {
    /// Convert from a single byte, which assumes an integer value (0-255).
    fn from(byte: u8) -> Self {
        Self(u16::from(byte) * 200)
    }
}
impl From<MPH> for f32 {
    /// Convert the [`MPH`] into a [`f32`] value.
    fn from(mph: MPH) -> Self {
        // FIXME(mdegans): use fractions in the future. It's likely cheaper and
        // the float usage is just begging for some accumulation errors -- like
        // Jeep may have done with the wildly innacurate MPG (at least on 4xE).
        f32::from(mph.0) / 200.0
    }
}
impl std::fmt::Display for MPH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: f32 = self.clone().into();
        f.write_fmt(format_args!("MPH({:.2})", value))
    }
}

impl TryFrom<Frame<Valid>> for MPH {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        if let Some(&value) = frame.data().get(7) {
            Ok(Self(value.into()))
        } else {
            Err(ParseError::Len {
                frame: frame.into(),
                expected: 8,
            })
        }
    }
}

/// The Jeep's speed in modern units.
pub struct KPH(pub(crate) MPH);
impl KPH {
    /// The raw u16 value, which is 200x the actual **MPH**. Do not use this
    /// expecting KPH.
    pub const fn raw(self) -> u16 {
        self.0.raw()
    }
}
impl From<MPH> for KPH {
    /// Convert from [`MPH`] to [`KPH`] losslessly.
    fn from(mph: MPH) -> Self {
        KPH(mph)
    }
}
impl From<KPH> for f32 {
    /// Convert from [`KPH`] into a [`f32`] value.
    fn from(kph: KPH) -> Self {
        f32::from(kph.0) * 1.60934
    }
}

/// The Jeep's [`Engine`] rpms.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct RPMs(pub(crate) u16);
impl RPMs {
    /// The raw [`u16`] value, where `0xffff` represents the engine being off.
    #[inline]
    pub const fn raw(self) -> u16 {
        self.0
    }
    /// Returns true if the engine is on.
    #[inline]
    pub const fn engine_is_on(self) -> bool {
        self.raw() != 0xffff
    }
    #[inline]
    /// Returns true if the engine is off
    pub const fn engine_is_off(self) -> bool {
        !self.engine_is_on()
    }
    #[inline]
    /// Get Some(rpms) if the engine is on, or None if the engine is off.
    pub const fn get(self) -> Option<u16> {
        if self.engine_is_on() {
            Some(self.0)
        } else {
            None
        }
    }
}
impl std::fmt::Display for RPMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Some(rpms) => f.write_fmt(format_args!("RPMs(Some({rpms}))")),
            None => f.write_str("RPMs(None)"),
        }
    }
}

/// Engine related Events
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
#[repr(align(8))]
pub enum Engine {
    /// Engine RPMs
    RPMs(RPMs),
    /// Current speed (Not GPS Corrected).
    ApproxMPH(MPH),
    /// Current speed (GPS corrected).
    MPH(MPH),
}

impl TryFrom<Frame<Valid>> for OneOrMany<Engine> {
    type Error = ParseError;

    fn try_from(frame: Frame<Valid>) -> Result<Self, Self::Error> {
        match frame.id() {
            0x340 => Ok(One(Engine::MPH(MPH::try_from(frame)?))),
            0x322 => {
                // the expected frame length
                const LEN: usize = 8;

                let mut engines = Vec::new();

                let data: [u8; LEN] = match frame.data().try_into() {
                    Ok(data) => data,
                    // frame does not match the expected length
                    Err(_) => {
                        return Err(ParseError::Len {
                            frame: frame.into(),
                            expected: LEN,
                        })
                    }
                };

                let rpms = RPMs(u16::from_be_bytes([data[0], data[1]]));
                engines.push(Engine::RPMs(rpms));

                // This is the approximate MPH, which is not GPS corrected.
                let mph = MPH(u16::from_be_bytes([data[2], data[3]]));
                engines.push(Engine::ApproxMPH(mph));

                // TODO(mdegans): investigate the last two bytes
                Ok(Many(engines))
            }
            _ => Err(ParseError::Id { frame }),
        }
    }
}

impl From<OneOrMany<Engine>> for OneOrMany<Event> {
    fn from(engine_or_engines: OneOrMany<Engine>) -> Self {
        match engine_or_engines {
            One(engine) => One(Event::Engine(engine)),
            Many(engines) => Many(
                engines
                    .into_iter()
                    .map(|engine| Event::Engine(engine))
                    .collect(),
            ),
        }
    }
}
