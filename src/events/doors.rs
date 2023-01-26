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
  /// [`bitflags`] representing the state of the jeep's [`Doors`].
  ///
  /// ### **Note**: use `are_all_closed` and `are_open_at_all` methods instead
  /// of `all` or `any`.
  #[cfg_attr(rustfmt, rustfmt_skip)]
  #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
  #[derive(Display)]
  #[repr(align(8))]
  pub struct Doors: u8 {
      // TEST_PASS(mdegans): 2021 4xE Sahara
      /// Bit for driver's side door.
      const DRIVER                    = 0b00000001;
      // TEST_PASS(mdegans): 2021 4xE Sahara
      /// Bit for passenger's side door.
      const PASSENGER                 = 0b00000010;
      // TEST_PASS(mdegans): 2021 4xE Sahara
      /// Bit for rear driver's side door.
      const REAR_DRIVER               = 0b00000100;
      // TEST_PASS(mdegans): 2021 4xE Sahara
      /// Bit for rear passenger's side door.
      const REAR_PASSENGER            = 0b00001000;
      /// This bit appears to be unused on the Wrangler but might represent a
      /// door on some models. If you can identify this, please send a PR
      /// with `MYSTERY_DOOR_0` changed to whatever it does represent.
      const MYSTERY_DOOR_0            = 0b00010000;
      // TEST_PASS(mdegans): 2021 4xE Sahara
      /// The rear swing gate (that usually holds the spare tire).
      const SWING_GATE                = 0b00100000;
      // TEST_PASS(mdegans): 2021 4xE Sahara - Behavior is as described:
      /// This bit is set when all doors are closed and locked, but if a door
      /// is opened from the inside it remains set leading to odd results like
      /// "Doors(PASSENGER | MYSTERY_DOOR_1)". Unlocking unsets all bits as
      /// expected. Not sure what exactly the purpose is or if it's a bug.
      /// This bit does not guarantee the doors are secure. More investigation
      /// is needed.
      const MYSTERY_BIT               = 0b01000000;
      /// This bit appears to be unused no the Jeep but might be on other
      /// models.
      const MYSTERY_DOOR_2            = 0b10000000;
      /// This is a mask for all the doors on the Jeep Wrangler. Ones
      /// represent doors present while zeroes are unused or unknown usage.
      const ALL_JEEP_DOORS            = 0b00101111;
  }
}

impl Doors {
    /// Returns true if all Jeep doors are closed.
    #[inline]
    pub const fn all_closed(self) -> bool {
        // basically (self & Self::ALL_JEEP_DOORS).bits == 0, and that works too
        self.intersection(Self::ALL_JEEP_DOORS).is_empty()
    }
    /// Returns true if any Jeep door is open.
    #[inline]
    pub const fn any_open(self) -> bool {
        !self.all_closed()
    }
}
