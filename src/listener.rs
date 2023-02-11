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

//! Contains a [`socketcan`]-powered event [`Listener`] to handle events from
//! a Linux socketcan interface. Requires the `socketcan` feature.

use derive_more::{Display, Error as DeriveError, From};
use socketcan::CANSocket;

use crate::{
    events::{self, Event, OneOrMany},
    frame::state::LenTooBig,
};
use OneOrMany::{Many, One};

/// An [`Error`] can be either an [`std::io::Error`] or a [`ParseError`]
#[derive(Debug, Display, DeriveError, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    #[cfg_attr(feature = "serde", serde(skip))]
    IoError(std::io::Error),
    /// Something went wrong converting input into a [`Frame`]
    InvalidInput(events::Error<LenTooBig>),
}

/// A [`Message`] is just a [`Result`] type produced by [`Listener`]'s methods.
// **NOTE(mdegans)**:I chose the name because it gave more information about
// the use case and most of the methods still make sense when you call them.
pub type Message = Result<Event, Error>;

/// An iterator through all waiting [`Event`] or [`Error`] from the [`Listener`].
pub struct Messages<'a> {
    sock: &'a CANSocket,
    pending: Vec<Event>,
}

/// An [`Iterator`] through [`Messages`] (`Vec<Result<Event, Error>>`) from the [`Listener`]
impl<'a> Iterator for Messages<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        // If there are pending events that have not been yielded, yield them
        // before doing any IO and getting an new frame.
        if !self.pending.is_empty() {
            // FIXME(mdegans): it's probably better if pending is a vector of
            // message since given a single frame, there can be some events that
            // parse sucessfully and some that do not, and it avoids the map so
            // this would become self.pending.pop() and the whole function
            // can get cleaner.
            return self.pending.pop().map(|event| Ok(event));
        }
        match self.sock.read_frame() {
            // We got a frame, so try to parse One or Many Events from it.
            Ok(frame) => match Event::parse(frame) {
                // Many events from a single CANFrame
                Ok(Many(events)) => {
                    self.pending = events;
                    // Unwrap here can never panic because the parsing code
                    // in every  `try_from` always returns at least one event
                    // inside a `Many` variant (unless that's broke).
                    Some(Ok(self.pending.pop().unwrap()))
                }
                // One `Event` from a single CANFrame
                Ok(One(event)) => Some(Ok(event)),
                // ParseError from a CANFrame
                Err(err) => Some(Err(err.into())),
            },
            // Some kind of IO error from `read_frame`
            Err(err) => match err.kind() {
                // Reading would block and we're set to non-blocking, so we're
                // done iterating for now (poll for some more messages later).
                std::io::ErrorKind::WouldBlock => None,
                // Any other IO error we wrap in an err. A simpler design just
                // returns None for any err, but then there's no way to tell the
                // difference between IOError and WouldBlock, and some IO errors
                // might be recoverable if the socket is still open.
                _ => Some(Err(err.into())),
            },
        }
    }
}

/// A Listener's job is to listen for CAN [`Messages`].
pub struct Listener {
    sock: CANSocket,
}

impl Listener {
    /// Connect the `Listener` to a can `interface` like `"can1"`.
    ///
    /// If `blocking` is true, the [`Messages`] iterator will block forever waiting
    /// for new messages.
    ///
    /// If `blocking` is false, the [`Messages`] iterator will never block and
    /// will terminate as soon as a socket read [`WouldBlock`](std::io::ErrorKind::WouldBlock).
    pub fn connect(
        interface: &str,
        blocking: bool,
    ) -> Result<Self, socketcan::CANSocketOpenError> {
        // Open CAN device socket
        let sock = CANSocket::open(interface)?;
        sock.set_nonblocking(!blocking)?;

        Ok(Listener { sock })
    }

    /// Iterate through all [`Event`] (or [`Error`]) waiting on the
    /// CAN bus. This iterator may be blocking or non-blocking depending on
    /// how the [`Listener`] was constructed.
    ///
    /// IO Errors, other than [`WouldBlock`](std::io::ErrorKind::WouldBlock)
    /// do not stop iteration. It's up to the caller to decide how to handle
    /// these since some IO Errors might be recoverable.
    pub fn messages<'a>(&'a self) -> Messages<'a> {
        Messages {
            sock: &self.sock,
            pending: Vec::new(),
        }
    }
}
