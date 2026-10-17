use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 100-nanosecond intervals between 1601-01-01 (the FILETIME epoch) and 1970-01-01.
const FILE_TIME_INTERVALS_BEFORE_UNIX_EPOCH: i64 = 116_444_736_000_000_000;
const NANOSECONDS_PER_FILE_TIME_INTERVAL: i64 = 100;

/// Converts a FILETIME, stored as one 64-bit count of 100 ns intervals, into `SystemTime`.
pub(super) fn system_time_from_file_time(file_time: i64) -> SystemTime {
    let intervals_since_unix_epoch =
        file_time.saturating_sub(FILE_TIME_INTERVALS_BEFORE_UNIX_EPOCH);
    let nanoseconds = intervals_since_unix_epoch.unsigned_abs()
        * NANOSECONDS_PER_FILE_TIME_INTERVAL.unsigned_abs();
    let offset = Duration::from_nanos(nanoseconds);

    if intervals_since_unix_epoch >= 0 {
        UNIX_EPOCH + offset
    } else {
        UNIX_EPOCH.checked_sub(offset).unwrap_or(UNIX_EPOCH)
    }
}

pub(super) fn system_time_from_file_time_parts(high: u32, low: u32) -> SystemTime {
    let file_time = (u64::from(high) << 32) | u64::from(low);
    system_time_from_file_time(file_time.cast_signed())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn converts_the_unix_epoch() {
        assert_eq!(
            system_time_from_file_time(FILE_TIME_INTERVALS_BEFORE_UNIX_EPOCH),
            UNIX_EPOCH
        );
    }

    #[test]
    fn keeps_sub_second_precision() {
        let one_and_a_half_seconds_later = FILE_TIME_INTERVALS_BEFORE_UNIX_EPOCH + 15_000_000;

        assert_eq!(
            system_time_from_file_time(one_and_a_half_seconds_later),
            UNIX_EPOCH + Duration::from_millis(1500)
        );
    }

    #[test]
    fn combines_high_and_low_parts() {
        let file_time = FILE_TIME_INTERVALS_BEFORE_UNIX_EPOCH.cast_unsigned();
        #[allow(clippy::cast_possible_truncation)]
        let (high, low) = ((file_time >> 32) as u32, file_time as u32);

        assert_eq!(system_time_from_file_time_parts(high, low), UNIX_EPOCH);
    }
}
