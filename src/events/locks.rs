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

use super::Display;

bitflags::bitflags! {
  /// State of the jeep's [`Locks`].
  #[cfg_attr(rustfmt, rustfmt_skip)]
  #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
  #[derive(Display)]
  #[repr(align(8))]
  pub struct Locks: u8 {
      // FIXME(mdegans): 2021 4xE Sahara - All these are wrong. More testing
      // is needed.
      const DRIVER                    = 0b00000001;
      const PASSENGER                 = 0b00000010;
      const REAR_DRIVER               = 0b00000100;
      const REAR_PASSENGER            = 0b00001000;
      const MYSTERY_DOOR_0            = 0b00010000;
      const SWING_GATE                = 0b00100000;
      const MYSTERY_DOOR_1            = 0b01000000;
      const MYSTERY_DOOR_2            = 0b10000000;
      const ALL_JEEP_DOORS            = 0b00101111;
  }
}

impl Locks {
    /// Returns true if all doors are locked.
    #[inline]
    pub const fn all_locked(self) -> bool {
        self.intersection(Self::ALL_JEEP_DOORS).is_empty()
    }
    /// Returns true if any door is unlocked.
    #[inline]
    pub const fn any_unlocked(self) -> bool {
        !self.all_locked()
    }
}
