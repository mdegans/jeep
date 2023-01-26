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

use super::{Frame, ParseError};

/// [`chrono::NaiveDateTime`] is used for [`DateTime`] rather than writing it from scratch.
pub use chrono::NaiveDateTime as DateTime;

impl TryFrom<Frame> for DateTime {
    type Error = ParseError;

    fn try_from(frame: Frame) -> Result<Self, Self::Error> {
        // the expected `frame.id` for this event.
        const ID: u32 = 0x350;
        // the expected frame length
        const LEN: usize = 8;

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

        let seconds = data[0];
        let minutes = data[1];
        let hours = data[2];
        let year = u16::from_be_bytes([data[3], data[4]]);
        let month = data[5];
        let day = data[6];

        let date = chrono::NaiveDate::from_ymd_opt(
            year.into(),
            month.into(),
            day.into(),
        )
        .ok_or_else(|| ParseError::Data {
            frame: frame.clone(),
            detail: "invalid date".to_owned(),
        })?;
        let datetime = date
            .and_hms_opt(hours.into(), minutes.into(), seconds.into())
            .ok_or_else(|| ParseError::Data {
                frame: frame.clone(),
                detail: "invalid time".to_owned(),
            })?;

        Ok(datetime)
    }
}

#[cfg(test)]
mod tests {
    use super::{DateTime, Frame};
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_correct_time() {
        let frame =
            Frame::from_id_data_len(0x350, [7, 34, 13, 7, 231, 1, 11, 1], 8)
                .unwrap();
        let dt = DateTime::try_from(frame).unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 11);
        assert_eq!(dt.hour(), 13);
        assert_eq!(dt.second(), 7);
    }
}
