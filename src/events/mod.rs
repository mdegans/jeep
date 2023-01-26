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

//! Events are sucessfully parsed frames from the CAN bus. The top level [`Event`]
//! enum contains sub-events like [`ControlPanel`](control_panel::ControlPanel)
//! which occasionally contain sub-events. Using it without a [`Listener`](crate::Listener)
//! (requires `socketcan` feature to be enabled), looks like this:
//!
//! ```
//! use jeep::{Frame, Event};
//! use jeep::events::{
//!     OneOrMany,
//!     OneOrMany::One,
//!     OneOrMany::Many,
//! };
//!
//! // Handler that prints only `Doors` events
//! fn handle(event: Event) {
//!     // or `match` if you're interested in more than one category
//!     if let Event::Doors(doors) = event {
//!         println!("{doors}");
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let data = (0x07000000_00000001_u64).to_be_bytes();
//! // Let's assume this is a valid frame...
//! let frame = Frame::from_id_data_len(0x2d3, data, 8)?;
//! // Here's one way to handle all events in that frame:
//! for event in Event::parse(frame)? {
//!     handle(event);
//! }
//!
//! /// .. which is more or less sugar for:
//! # let frame = Frame::from_id_data_len(0x2d3, data, 8)?;
//! match OneOrMany::<Event>::try_from(frame)? {
//!     One(event) => handle(event),
//!     Many(events) => {
//!         for event in events {
//!             handle(event);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Matching [`Event`] is the primary way to filter out what you're looking for.
//! Another is to try to convert a [`Frame`] directly into a sub-event.
//!
//! ```
//! use jeep::{Frame, events::control_panel::Buttons};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let data = (0x07000000_00000001_u64).to_be_bytes();
//! // Let's assume this is a valid frame...
//! let frame = Frame::from_id_data_len(0x2d3, data, 8)?;
//! // ... and you only care about `control_panel::Buttons`, you can try to
//! // parse a frame directly into a sub-event like this:
//! let parsed = Buttons::try_from(frame)?;
//! # Ok(())
//! # }
//! ```
//!
//! Parsing a sub-event directly is more effecient, but for most purposes this
//! won't be noticable. See [`ParseError`] for handling unrecognized
//! and invalid frames.

use derive_more::{Display, From};
use static_assertions as sa;

use crate::Frame;

mod parse_error;
pub use parse_error::ParseError;

pub mod battery;
pub mod bus;
pub mod camera;
pub mod control_panel;
pub mod datetime;
pub mod doors;
pub mod engine;
pub mod force;
pub mod hvac;
pub mod ignition;
pub mod lights;
pub mod locks;
pub mod odometer;
pub mod remote;
pub mod steering_wheel;

use FrontOrRear::{Front, Rear};
use OneOrMany::{Many, One};

// This shouldn't change for performance's sake. An subevent should probly not
// be > 12 in size and alignment ideally 4 or 8, preferably 8. Use heap allocation
// if somehow your event is huge (like a utf-8 text concatenation). Since String
// and Vec<u8> are 24 in size, it might be necessary to raise this size check to
// 32 at some point. Use powers of two for size. Alignment should remain at 8.
sa::const_assert_eq!(std::mem::size_of::<Event>(), 16);
sa::const_assert_eq!(std::mem::align_of::<Event>(), 8);
// ControlPanel is the only one that's 16 and that's allowed because **magic**.
// It's the only one allowed to have a size of 16. In the future it might
// shrink, but the Event itself will always be size 16 align 8.
sa::const_assert_eq!(std::mem::size_of::<control_panel::ControlPanel>(), 16);
sa::const_assert_eq!(std::mem::align_of::<control_panel::ControlPanel>(), 8);

/// Top-level Jeep [`Event`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, From, Clone)]
pub enum Event {
    /// [`battery::Battery`] related event (charge, etc.)
    Battery(battery::Battery),
    /// [`remote::Remote`] control event (KeyFob, App).
    Remote(remote::Remote),
    /// [`ignition::Ignition`] event from the ignition switch.
    Ignition(ignition::Ignition),
    /// One or more [`steering_wheel::Buttons`] were pressed.
    SteeringWheel(steering_wheel::Buttons),
    /// One or more [`control_panel::Buttons`] (below the head unit) were pressed.
    ControlPanel(control_panel::ControlPanel),
    /// [`lights::Lights`] (hazards, blinkers, cabin).
    Lights(lights::Lights),
    /// [`doors::Doors`] (state of all doors).
    Doors(doors::Doors),
    /// [`locks::Locks`] (state of all locks).
    Locks(locks::Locks),
    /// [`force::Force`] event (accelerometers, etc.)
    Force(force::Force),
    /// [`camera::Camera`] event
    Camera(camera::Camera),
    /// [`engine::Engine`] event (speed, temperatures under the hood etc.)
    Engine(engine::Engine),
    /// [`hvac::HVAC`] event (cabin temp, outside temp, fan speeds). (no
    /// [`engine::Engine`] temps)
    HVAC(hvac::HVAC),
    /// [`datetime::DateTime`] event (Jeep's reported date and time)
    DateTime(datetime::DateTime),
    /// [`odometer::Odometer`] event.
    Odometer(odometer::Odometer),
    /// [`bus::Bus`] event.
    Bus(bus::Bus),
}

impl Event {
    /// Parse [`OneOrMany<Event>`] from compatible input.
    ///
    /// As of writing that includes:
    /// * [`libc::can_frame`] - is always supported.
    /// * [`socketcan::CANFrame`] - if the `socketcan` feature is enabled.
    #[inline(always)] // because single function call
    pub fn parse<I, E>(input: I) -> Result<OneOrMany<Event>, E>
    where
        I: TryInto<OneOrMany<Event>, Error = E>,
    {
        input.try_into()
    }
}

/// Represents [`One`] or [`Many`] things.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
pub enum OneOrMany<T> {
    /// One `T`
    One(T),
    // TODO(mdegans): Use tinyvec or smallvec or something to avoid heap
    // allocation entirely. It'll make `OneOrMany` larger, but also faster.
    /// Many `T`'s
    Many(Vec<T>),
}

impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(Some(self))
    }
}

pub struct IntoIter<T>(Option<OneOrMany<T>>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let (elem, next) = match self.0.take() {
            Some(oom) => match oom {
                // We have one element to yield, nothing is next.
                One(elem) => (Some(elem), None),
                // We pop one elem to yield, the vec is still next, if it's
                // empty, next time it will pop None. This is correct.
                Many(mut vec) => (vec.pop(), Some(Many(vec))),
            },
            None => (None, None),
        };

        self.0 = next;

        elem
    }
}

impl TryFrom<Frame> for OneOrMany<Event> {
    type Error = ParseError;

    /// Parse a CAN frame into [`OneOrMany<Event>`]
    fn try_from(frame: Frame) -> Result<Self, Self::Error> {
        match frame.id() {
            0x2c2 => Ok(One(Event::Battery(frame.try_into()?))),
            0x1c0 => Ok(One(Event::Remote(frame.try_into()?))),
            0x122 => Ok(One(Event::Ignition(frame.try_into()?))),
            0x318 => Ok(One(Event::SteeringWheel(frame.try_into()?))),
            0x2d3 | 0x2d4 | 0x273 => {
                Ok(One(Event::ControlPanel(frame.try_into()?)))
            }
            // lights / locks / doors (multiple events come from this source in
            // a single frame)
            0x2fa => {
                // 0x2fa is the odd one out with multiple message catgegories
                // from the same source, so this fucker here is the entire
                // reason for the Many variant and heap allocation.
                // Otherwise events live on the stack. And as a result the
                // Messages iterator has to be more complex than otherwise, but
                // the flexibility is probably a good idea anyway.

                // the expected frame length
                const LEN: usize = 8;

                // FIXME(mdegans): these should be moved somewher else, and they
                // don't seem to work, which means some more time in the jeep.
                let mut events = Vec::new();
                let mut errors = Vec::new();

                let data: [u8; LEN] = match frame.data().try_into() {
                    Ok(data) => data,
                    Err(_) => {
                        return Err(ParseError::Len {
                            frame,
                            expected: LEN,
                        })
                    }
                };

                // unwrap can never panic since every bit has a flag
                let doors = doors::Doors::from_bits(data[0]).unwrap();
                events.push(Event::Doors(doors));

                // NOTE(mdegans): This and `Dimmer` does not work on my 4xE. Not
                // sure why. I find it odd parking lights would be in this frame.
                match lights::ParkingLights::try_from(frame.clone()) {
                    Ok(parking_lights) => events.push(Event::Lights(
                        lights::Lights::ParkingLights(parking_lights),
                    )),
                    Err(parse_error) => errors.push(parse_error),
                };

                match lights::Dimmer::try_from(frame.clone()) {
                    Ok(dimmer) => events
                        .push(Event::Lights(lights::Lights::Dimmer(dimmer))),
                    // somehow the dimmer value was out of range
                    Err(parse_error) => errors.push(parse_error),
                };

                // unwrap can never panic since every bit has a flag
                // FIXME(mdegans): It's unclear if locks are actually bitflags.
                // More investigation is needed.
                let locks = locks::Locks::from_bits(frame.data()[3]).unwrap();
                events.push(Event::Locks(locks));

                if errors.is_empty() {
                    Ok(Many(events))
                } else {
                    // FIXME(make ParseError support multiple errors. At least
                    // these won't pass silently for now.
                    Err(ParseError::Data {
                        frame,
                        detail: format!("There were error(s) parsing a frame from `0x2fa`: {errors:?}"),
                    })
                }
            }
            // Force sensors
            0x24e | 0x252 => Ok(One(Event::Force(frame.try_into()?))),
            // Camera view (from uconnect?)
            0x302 => Ok(One(Event::Camera(frame.try_into()?))),
            // Engine event (RPMs, MPH, Adjusted MPH)
            0x322 | 0x340 => {
                Ok(OneOrMany::<engine::Engine>::try_from(frame)?.into())
            }
            // HVAC event
            0x33a => Ok(One(Event::HVAC(frame.try_into()?))),
            // Vehicle date and time (TODO(mdegans): bus id?)
            0x350 => Ok(One(Event::DateTime(frame.try_into()?))),
            0x3d2 => Ok(One(Event::Odometer(frame.try_into()?))),
            // 0x4xx series messages
            0x401 => Ok(One(Event::Bus(frame.try_into()?))),
            // Something to implement.
            _ => Err(ParseError::Id { frame }),
        }
    }
}

#[cfg(feature = "socketcan")]
impl TryFrom<socketcan::CANFrame> for OneOrMany<Event> {
    type Error = CanFrameError;

    fn try_from(frame: socketcan::CANFrame) -> Result<Self, Self::Error> {
        let frame = Frame::from_socketcan(frame)?;
        // ? is not working here :/  It should because a `From` impl exists
        frame.try_into().map_err(|pe| CanFrameError::ParseError(pe))
    }
}

impl TryFrom<libc::can_frame> for OneOrMany<Event> {
    type Error = CanFrameError;

    fn try_from(frame: libc::can_frame) -> Result<Self, Self::Error> {
        let frame = Frame::from_libc_can_frame(frame)?;
        frame.try_into().map_err(|pe| CanFrameError::ParseError(pe))
    }
}

/// A [`Front`] or [`Rear`] thing.
// NOTE(mdegans):This is only used in one place. Maybe it's not as useful as I
// thought it would be.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Display, Clone)]
pub enum FrontOrRear<T> {
    Front(T),
    Rear(T),
}

/// Everything that can go wrong converting a CAN frame to [`OneOrMany<Event>`]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Debug, derive_more::Error, derive_more::From, derive_more::Display,
)]
pub enum CanFrameError {
    /// Len (`can_d) > 8. Constructing a [`Frame`] from this data would likely result in UB.
    BadLen(crate::frame::BadLen),
    /// Input could be converted into a [`Frame`] but something about it did not parse.
    ParseError(ParseError),
}
