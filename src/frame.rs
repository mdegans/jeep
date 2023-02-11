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

//! Contains our own CAN [`Frame`], which wraps a [`libc::can_frame`].

use static_assertions as sa;

use std::marker::PhantomData;

use self::state::{DataSafe, LenTooBig, LenUnexpected, Raw, Valid};

// Some compile time sanity checks to ensure socketcan and can_frame haven't
// changed somehow. These should probably never break.
sa::const_assert_eq!(std::mem::size_of::<libc::can_frame>(), 16);
#[cfg(feature = "socketcan")]
sa::assert_eq_size!(libc::can_frame, socketcan::CANFrame);
// note: socketcan alignment is not the same, however the field order and size
// still is.
sa::assert_eq_size!(libc::can_frame, Frame<Valid>);
sa::assert_eq_size!(libc::can_frame, Frame<LenTooBig>);
sa::assert_eq_size!(libc::can_frame, Frame<Raw>);
sa::assert_eq_size!(libc::can_frame, CanFrameWrapper);
sa::assert_eq_align!(libc::can_frame, CanFrameWrapper);
const DATA_MAX_LEN: usize = 8;

/// A [`Frame`] is a wrapper for a [`libc::can_frame`] struct.
///
/// It is guaranteed to have the same size and layout. This will not change.
// Class invariants:
// 1) self.0.can_dlc <= 8 - necessary for data() slice accessor.
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct Frame<State> {
    #[cfg_attr(feature = "serde", serde(with = "CanFrameWrapper"))]
    #[cfg_attr(feature = "serde", serde(flatten))]
    can_frame: libc::can_frame,
    #[cfg_attr(feature = "serde", serde(skip))]
    state: PhantomData<State>,
}

/// Represents the state of a Frame as zero sized types. I read about this in
/// Rust for Rustacians. We're "abusing" (brilliantly) the type system here to
/// tag structs with zero sized states so an object can be in one state or
/// another, but not both -- although Frame is so generic, so it could even
/// be a combination of zsts
///
/// This is basically an enum, but the tag is really zero sized. And the
/// *really* cool thing you can do is with implementations for individual
/// states. So, I'm using this for validation and making my impl into a state
/// machine.
pub mod state {
    /// Represents a raw, unparsed, state. All constructors be here which can
    /// progress onto either a Validated or some failure state.
    pub struct Raw;

    /// Represents a valid state for a CAN Frame. Validation has happened after
    /// any mutation.
    #[derive(Clone)]
    pub struct Valid;
    /// Represents an invalid frame data len (`can_dlc > 8`).
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(derive_more::Display, Debug, derive_more::Error)]
    #[display = "Len (`can_dlc`) was > 8"]
    pub struct LenTooBig;

    /// Represents an unexpected frame length for a given CAN Id.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(derive_more::Display, Debug, derive_more::Error)]
    #[display = "Len of CAN Frame was unexpected for it's source ID."]
    pub struct LenUnexpected;

    /// Marks states where it's safe to access `.data()`. Otherwise using the
    /// data accessor would be unsafe.
    pub trait DataSafe {}
    /// It is safe to access `data()` when the state is `Valid` and `validate()`
    /// has returned a [`Frame<Valid>`].
    ///
    /// [Frame<Valid>]: super::Frame<Valid>
    impl DataSafe for Valid {}
    /// It is safe to access `data()` when the len is unexpected, since it's
    /// still validated to be < 8.
    impl DataSafe for LenUnexpected {}
}

/// A [`Raw`] frame only construction methods. One can become either a Valid
/// or one of various invalid frames depending on what goes wrong.
impl Frame<Raw> {
    /// Create a new, zeroed out Self.
    const fn zeroed() -> Self {
        // SAFETY: there is no "safe" way to construct a libc::can_frame with
        // private fields, and zeroing out a struct is the proper way to do so.
        // std::mem::zeroed() is not const (yet), but we can use transmute.
        // Transmute is safe because zeroes transmuted into a libc::can_frame is
        // valid for it's type.
        unsafe {
            std::mem::transmute([0u8; std::mem::size_of::<libc::can_frame>()])
        }
    }

    /// Helper function to validate that a [`Frame`] is validly constructed.
    /// **All constructors must call this!** (in order to maintain class
    /// invariant 1, which avoids a panic). Serde deserialization calls
    ///
    const fn validate(self) -> Result<Frame<Valid>, Frame<LenTooBig>> {
        let Self {
            can_frame: frame, ..
        } = self;
        // Ensure len does not exceed the maximum len of data.
        if frame.can_dlc <= DATA_MAX_LEN as u8 {
            Ok(Frame {
                can_frame: frame,
                state: PhantomData,
            })
        } else {
            Err(Frame {
                can_frame: frame,
                state: PhantomData,
            })
        }
    }

    /// Create a new [`Frame`] from a [`libc::can_frame`].
    #[inline(always)] // because trivial
    pub const fn from_libc_can_frame(
        can_frame: libc::can_frame,
    ) -> Result<Frame<Valid>, Frame<LenTooBig>> {
        Frame {
            can_frame,
            state: PhantomData,
        }
        .validate()
    }

    /// Create a new frame from id (with flags), data, and len.
    pub const fn from_id_data_len(
        id_flags: u32,
        data: [u8; 8],
        len: u8,
    ) -> Result<Frame<Valid>, Frame<LenTooBig>> {
        let mut frame = Frame::zeroed();

        frame.can_frame.can_id = id_flags;
        frame.can_frame.can_dlc = len;
        frame.can_frame.data = data;

        frame.validate()
    }

    /// Create a new frame from id_flags and a data slice.
    pub const fn from_id_slice(
        id_flags: u32,
        slice: &[u8],
    ) -> Result<Frame<Valid>, LenTooBig> {
        if slice.len() > 8 {
            // we can't possibly make frame data from the slice, so we just
            return Err(LenTooBig);
        }

        let len: u8 = slice.len() as u8;
        let mut data = [0u8; 8];

        let mut i = 0;
        while i < slice.len() {
            data[i] = slice[i];
            i += 1
        }

        match Self::from_id_data_len(id_flags, data, len) {
            Ok(frame) => Ok(frame),
            Err(_) => Err(LenTooBig),
        }
    }

    /// Create a new [`Frame`] from a [`socketcan::CANFrame`].
    #[inline] // because single function call
    #[cfg(feature = "socketcan")]
    pub fn from_socketcan(
        frame: socketcan::CANFrame,
    ) -> Result<Frame<Valid>, LenTooBig> {
        // TODO(mdegans): looks like socketcan's "master" branch is also wrapping
        // a libc::can_frame, so in the future we can probably bypass this constructor
        // and just check the len.
        Self::from_id_slice(frame.id(), frame.data())
    }
}

impl<State> Frame<State>
where
    State: DataSafe,
{
    /// CAN frame's data as slice.
    #[inline] // because trivial accessor (in release)
    pub const fn data(&self) -> &[u8] {
        // SAFETY: Class invariant 1 guarantees Len is valid, and the dcheck
        // below will check that in debug builds.
        debug_assert!(
                self.can_frame.can_dlc <= DATA_MAX_LEN as u8,
                "Class invariant 1 violated. `can_dlc` (the `len`) is > Self::DATA_LEN"
            );
        unsafe {
            core::slice::from_raw_parts(
                &self.can_frame.data as *const u8,
                self.can_frame.can_dlc as usize,
            )
        }
    }
}

impl<State> Frame<State> {
    const ID_MASK: u32 = 0x7FF;

    /// The Id (masked by [`Frame::ID_MASK`]) from which the Frame was sent.
    #[inline] // because trivial accessor
    pub const fn id(&self) -> u32 {
        self.raw_id() & Self::ID_MASK
    }

    /// The raw `can_id` of the [`can_frame`]
    ///
    /// [`can_frame`]: libc::can_frame
    #[inline] // because trivial accessor
    pub const fn raw_id(&self) -> u32 {
        self.can_frame.can_id
    }
}

impl Frame<Valid> {
    #[inline] // because trivial
    pub const fn into_libc_can_frame(self) -> libc::can_frame {
        self.can_frame
    }

    /// Convert into a [`socketcan::CANFrame`]
    #[inline] // because single function call
    #[cfg(feature = "socketcan")]
    pub fn into_socketcan(
        self,
    ) -> Result<socketcan::CANFrame, socketcan::ConstructionError> {
        // TODO(mdegans): looks like socketcan's "master" branch is also wrapping
        // a libc::can_frame, so in the future we can probably bypass this constructor.
        socketcan::CANFrame::new(self.id(), self.data(), false, false)
    }
}

impl core::hash::Hash for Frame<Valid> {
    /// This implementation of hash ignores any padding to avoid, for example,
    /// "duplicate" frames in a collection that differ.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.can_frame.can_id.hash(state);
        self.can_frame.can_dlc.hash(state);
        self.data().hash(state);
    }
}

impl PartialEq for Frame<Valid> {
    fn eq(&self, other: &Self) -> bool {
        self.can_frame.can_id == other.can_frame.can_id
            && self.can_frame.can_dlc == other.can_frame.can_dlc
            && self.data() == other.data()
    }
}

#[cfg(feature = "embedded-can")]
impl embedded_can::Frame for Frame<Valid> {
    fn new(id: impl Into<embedded_can::Id>, data: &[u8]) -> Option<Self> {
        let id: embedded_can::Id = id.into();
        match id {
            embedded_can::Id::Standard(id) => {
                Frame::from_id_slice(id.as_raw().into(), data).ok()
            }
            // We should not be getting Extended frames on the Jeep JL
            embedded_can::Id::Extended(_) => None,
        }
    }

    // Not implemented for the `jeep` crate. Will always return None.
    #[inline(always)] // because trivial constant
    fn new_remote(_: impl Into<embedded_can::Id>, __: usize) -> Option<Self> {
        None
    }

    #[inline(always)] // because trivial constant
    fn is_extended(&self) -> bool {
        false
    }

    #[inline(always)] // because trivial constant
    fn is_remote_frame(&self) -> bool {
        false
    }

    fn id(&self) -> embedded_can::Id {
        // Unwrap can never panic because the id() accessor always returns a masked out id
        embedded_can::Id::Standard(
            embedded_can::StandardId::new(self.id().try_into().unwrap())
                .unwrap(),
        )
    }

    fn dlc(&self) -> usize {
        self.can_frame.can_dlc.into()
    }

    fn data(&self) -> &[u8] {
        self.data()
    }
}

impl std::fmt::Debug for Frame<LenTooBig> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, stringify!(Frame<LenInvalid>))
    }
}

impl std::fmt::Display for Frame<LenTooBig> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(&self, f)
    }
}

impl std::error::Error for Frame<LenTooBig> {}

impl<State> std::fmt::Debug for Frame<State>
where
    State: DataSafe,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // we're "lying" here, but it's prettier.
        f.debug_struct(stringify!(CanFrame))
            .field("id", &self.id())
            .field("data", &self.data())
            .finish()
    }
}

impl<State> std::fmt::Display for Frame<State>
where
    State: DataSafe,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:3X}#{:X?}", self.id(), self.data())
    }
}

#[cfg(feature = "socketcan")]
impl TryFrom<socketcan::CANFrame> for Frame<Valid> {
    type Error = LenTooBig;

    #[inline(always)] // because single function call
    fn try_from(frame: socketcan::CANFrame) -> Result<Self, Self::Error> {
        Frame::from_socketcan(frame)
    }
}

/// Exists because serde's derive macros demand it for remote private fields,
/// even when it's skip, and even when a conversion exists. Bug?
#[inline(always)] // because we want 0 inlined everywhere this is called
#[cfg(feature = "serde")]
const fn _zero(_frame: &libc::can_frame) -> u8 {
    0
}

/// A custom deserializer for the frame's len value
#[cfg(feature = "serde")]
#[inline(always)] // because only used once
fn deserialize_len<'de, D, const MAX: u8>(d: D) -> Result<u8, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::Deserialize;

    let len = u8::deserialize(d)?;
    if len <= MAX {
        Ok(len)
    } else {
        Err(serde::de::Error::custom(LenTooBig))
    }
}

#[cfg(feature = "serde")]
#[inline(always)] // because only used once
fn deserialize_len_8<'de, D>(d: D) -> Result<u8, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    deserialize_len::<'de, D, 8>(d)
}

/// A Wrapper for a [`libc::can_frame`] that enables serialization
#[repr(C, align(8))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(remote = "libc::can_frame"))]
struct CanFrameWrapper {
    /// The raw Id + flags first member of the
    can_id: u32,
    /// Class invariant: this is guaranteed by all constructors to be between
    /// 0 and 8
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "deserialize_len_8")
    )]
    can_dlc: u8,
    /// Padding/reserved. Ignored by Serialize, Deserialize, Hash, PartialEq, etc.
    // `skip` alone doesn't work, unfortunately, so we need a fake getter to inline 0
    // at the callsites and then it works.
    #[cfg_attr(feature = "serde", serde(skip, getter = "_zero"))]
    __pad: u8,
    #[cfg_attr(feature = "serde", serde(skip, getter = "_zero"))]
    __res0: u8,
    #[cfg_attr(feature = "serde", serde(skip, getter = "_zero"))]
    __res1: u8,
    /// The CAN payload data. Bytes past `len` are invalid, but will be
    /// Serialized and Deserialized if that feature is enabled.
    ///
    /// **NOTE**: Ugh. Default serialization of this results in the entire data
    /// structure being deserialized, including the unused space. Some
    /// serialization formats it might not matter for, and might even benefit,
    /// but it cannot be relied on that the hash of the serialized data will be
    /// unique vs our implementation of Hash and PartialEq which ignores unused
    /// data.
    data: [u8; 8],
}

impl CanFrameWrapper {
    #[inline(always)] // because single function call
    pub const fn into_libc_can_frame(self) -> libc::can_frame {
        // SAFETY: The compiler guarantees the size is the same and serde's
        // `remote_type` guarantees the layout is the same.
        // Both structs are repr(C)
        unsafe { std::mem::transmute(self) }
    }
}

impl From<CanFrameWrapper> for libc::can_frame {
    #[inline(always)] // because single function call
    fn from(frame: CanFrameWrapper) -> Self {
        frame.into_libc_can_frame()
    }
}

impl From<Frame<Valid>> for Frame<LenUnexpected> {
    /// Asserts that this type's length is invalid. It's not possible to
    /// convert the other direction.
    fn from(frame: Frame<Valid>) -> Self {
        Self {
            can_frame: frame.can_frame,
            state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Frame;

    #[test]
    fn test_from_libc() {
        // SAFETY: Zeroing out the struct is the proper way to construct a
        // can_frame.
        let mut libc_frame: libc::can_frame = unsafe { std::mem::zeroed() };
        libc_frame.can_id = 1;
        libc_frame.can_dlc = 3;
        libc_frame.data = [2, 3, 4, 5, 6, 7, 8, 9];
        let frame = Frame::from_libc_can_frame(libc_frame.clone()).unwrap();
        assert_eq!(frame.id(), libc_frame.can_id);
        assert_eq!(
            frame.data(),
            &libc_frame.data[0..libc_frame.can_dlc as usize]
        )
    }

    #[test]
    #[cfg(feature = "socketcan")]
    fn test_from_socketcan() {
        let sc_frame =
            socketcan::CANFrame::new(1, &[2, 3, 4], false, false).unwrap();
        let frame = Frame::from_socketcan(sc_frame.clone()).unwrap();
        assert_eq!(frame.id(), sc_frame.id());
        assert_eq!(frame.data(), sc_frame.data());
    }

    #[test]
    fn test_data() {
        let frame =
            Frame::from_id_data_len(1, [2, 3, 4, 5, 6, 7, 8, 9], 2).unwrap();
        assert_eq!(frame.data(), &[2, 3]);
    }

    #[test]
    fn test_validate_len() {
        let ret = Frame::from_id_data_len(1, [2, 3, 4, 5, 6, 7, 8, 9], 255);
        assert!(ret.is_err())
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_frame_serde_json() {
        use crate::frame::state::Valid;

        let expected =
            Frame::from_id_data_len(1, [2, 3, 4, 5, 6, 7, 8, 9], 8).unwrap();
        let json = serde_json::to_string(&expected).unwrap();
        assert_eq!(
            &json,
            "{\"can_id\":1,\"can_dlc\":8,\"data\":[2,3,4,5,6,7,8,9]}"
        );
        let actual: Frame<Valid> = serde_json::from_str(&json).unwrap();
        assert_eq!(actual, expected);

        // Ensure class invariant 1 is upheld even with deserialization of bad len
        const BAD_LEN: &str =
            "{\"can_id\":1,\"can_dlc\":9,\"data\":[2,3,4,5,6,7,8,9]}";
        let err = serde_json::from_str::<Frame<Valid>>(BAD_LEN).unwrap_err();
        assert_eq!(err.to_string(), "LenTooBig at line 1 column 49");

        // Data being too long should also fail.
        const BAD_DATA: &str =
            "{\"can_id\":1,\"can_dlc\":8,\"data\":[2,3,4,5,6,7,8,9,10]}";
        let err = serde_json::from_str::<Frame<Valid>>(BAD_DATA).unwrap_err();
        assert_eq!(err.to_string(), "invalid length 9, expected 8 elements in sequence at line 1 column 52");
    }
}
