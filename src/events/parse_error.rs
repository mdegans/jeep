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

use crate::frame::Frame;

/// When an [`Event`](super::Event) fails to parse from a [`Frame`]. It is
/// convertible back into a [`Frame`] using [`ParseError::into()`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub enum ParseError {
    /// [`Frame`] ID was unrecognized.
    Id {
        /// The frame with the unrecognized ID.
        frame: Frame,
    },
    /// [`Frame`]'s len (`can_dlc`) was not the expected length from the ID.
    Len {
        /// The frame that failed to parse.
        frame: Frame,
        /// The expected length for this id.
        expected: usize,
    },
    /// Invalid data in [`Frame`] with detail.
    Data {
        /// The frame that failed to parse or None if it was unsafe to construct a Frame.
        frame: Frame,
        /// Why the frame failed to parse (too big, too small, etc...)
        detail: String,
    },
}

impl Into<Frame> for ParseError {
    /// Convert a [`ParseError`] back into the [`Frame`] that failed to parse.
    fn into(self) -> Frame {
        match self {
            ParseError::Id { frame }
            | ParseError::Len { frame, .. }
            | ParseError::Data { frame, .. } => frame,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Id { frame } => write!(
                f,
                "Frame id `{:#X}` not recognized (data: `{:#X?}`).",
                frame.id(),
                frame.data(),
            ),
            ParseError::Len { frame, expected } => write!(
                f,
                "Frame's length ({}); unexpected from source `{:#X}` (expected: {}).",
                frame.data().len(),
                frame.id(),
                expected,
            ),
            ParseError::Data { frame, detail } => write!(
                f,
                "Frame from source id `{:#X}` with data `{:#X?}` failed validation because: {}",
                frame.id(),
                frame.data(),
                detail,
            ),
        }
    }
}

// TODO(mdegans): add sources. (OneOrMany?)
impl std::error::Error for ParseError
where
    ParseError: std::fmt::Display + core::fmt::Debug,
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    #[inline]
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}
