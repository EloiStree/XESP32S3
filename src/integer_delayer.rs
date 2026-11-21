//! Integer delayer module for scheduling delayed execution of integer values.
//!
//! This module allows scheduling integer values to be executed at specific times in the future,
//! with support for millisecond precision delays. It includes a check to ensure UDP received
//! values are divisible by 16 (indicating valid IID format packets).

use heapless::Vec;

/// Maximum number of delayed integers that can be queued
const MAX_DELAYED_VALUES: usize = 256;

/// Holds an integer value and the time at which it should be executed
#[derive(Clone, Copy, Debug)]
pub struct IndexValueIntegerToDelayValue {
    pub m_index: u32,
    pub m_integer: u32,
    pub m_when_to_execute: u64,
}

/// Delays integer values to their requested execution time (milliseconds)
///
/// This allows precise scheduling of macro actions by receiving all instructions
/// in one block and executing them at their scheduled millisecond precision times.
pub struct IndexValueIntegerTimeMillisecondsDelayer {
    delays_integer: Vec<IndexValueIntegerToDelayValue, MAX_DELAYED_VALUES>,
    rtc_time_ms: u64,
    tz_offset_ms: i64,
}

impl IndexValueIntegerTimeMillisecondsDelayer {
    /// Creates a new delayer instance
    ///
    /// # Arguments
    /// * `rtc_time_seconds` - Initial RTC time in seconds
    /// * `timezone_offset_seconds` - Timezone offset in seconds
    pub const fn new(rtc_time_seconds: f64, timezone_offset_seconds: f64) -> Self {
        Self {
            delays_integer: Vec::new(),
            rtc_time_ms: (rtc_time_seconds * 1000.0) as u64,
            tz_offset_ms: (timezone_offset_seconds * 1000.0) as i64,
        }
    }

    /// Checks if UDP payload is valid for processing (divisible by 16, not 1024)
    ///
    /// If the array is 1024 bytes, it's a screen update.
    /// If the array is divisible by 16, it's valid IID format:
    /// - Index: 4 bytes (u32)
    /// - Value: 4 bytes (u32)
    /// - Delay time: 8 bytes (u64)
    ///
    /// If the delay value is under 36,000,000 ms, it's treated as milliseconds delay.
    pub fn append_value_if_dividable_by_16_and_not_1024(&mut self, data: &[u8]) -> bool {
        let len = data.len();

        // Reject screen updates (1024 bytes)
        if len == 1024 {
            return false;
        }

        // Check if divisible by 16
        if len % 16 != 0 {
            return false;
        }

        // Process payload in 16-byte chunks
        // Format: 4-byte index + 4-byte value + 8-byte delay
        for chunk in data.chunks_exact(16) {
            if let (Ok(idx_bytes), Ok(val_bytes), Ok(delay_bytes)) = (
                <[u8; 4]>::try_from(&chunk[0..4]),
                <[u8; 4]>::try_from(&chunk[4..8]),
                <[u8; 8]>::try_from(&chunk[8..16]),
            ) {
                let index = u32::from_le_bytes(idx_bytes);
                let value = u32::from_le_bytes(val_bytes);
                let delay_ms = u64::from_le_bytes(delay_bytes);

                // Only process if delay is reasonable (under ~10 hours)
                if delay_ms < 36_000_000 {
                    let _ = self.append_integer_to_delay_as_milliseconds(index, value, delay_ms as i64);
                }
            }
        }

        true
    }

    /// Gets the current time in milliseconds
    pub fn get_current_time_on_pico_2w(&self) -> u64 {
        self.get_local_timestamp_in_milliseconds()
    }

    /// Gets the local timestamp in milliseconds with RTC and timezone offset
    pub fn get_local_timestamp_in_milliseconds(&self) -> u64 {
        let adjusted = self.rtc_time_ms as i64 + self.tz_offset_ms;
        (adjusted.max(0)) as u64
    }

    /// Appends an integer to the delay queue with a delay in milliseconds
    pub fn append_integer_to_delay_as_milliseconds(
        &mut self,
        integer_index: u32,
        integer_value: u32,
        milliseconds: i64,
    ) -> bool {
        if milliseconds < 0 {
            return false;
        }

        let current_time = self.get_local_timestamp_in_milliseconds();
        let delay_value = IndexValueIntegerToDelayValue {
            m_index: integer_index,
            m_integer: integer_value,
            m_when_to_execute: current_time + (milliseconds as u64),
        };

        // Insert at the front for newer items (reverse chronological order)
        self.delays_integer.insert(0, delay_value).is_ok()
    }

    /// Appends an integer with an explicit start time and additional delay
    pub fn append_integer_to_delay_as_milliseconds_from_time(
        &mut self,
        integer_index: u32,
        integer_value: u32,
        start_time: u64,
        milliseconds: i64,
    ) -> bool {
        if milliseconds < 0 {
            return false;
        }

        let delay_value = IndexValueIntegerToDelayValue {
            m_index: integer_index,
            m_integer: integer_value,
            m_when_to_execute: start_time + (milliseconds as u64),
        };

        self.delays_integer.insert(0, delay_value).is_ok()
    }

    /// Checks for integers ready to execute and returns them
    pub fn check_for_integer_to_execute(&mut self) -> Vec<IndexValueIntegerToDelayValue, 256> {
        let mut ready = Vec::new();
        let current_time = self.get_local_timestamp_in_milliseconds();

        // Iterate in reverse to safely remove items while iterating
        let mut i = self.delays_integer.len();
        while i > 0 {
            i -= 1;
            if self.delays_integer[i].m_when_to_execute <= current_time {
                let delay_value = self.delays_integer.remove(i);
                let _ = ready.push(delay_value);
            }
        }

        ready
    }

    /// Returns the number of delayed values currently queued
    pub fn queue_length(&self) -> usize {
        self.delays_integer.len()
    }
}
