use std::time::{Duration, SystemTime};

/// Seconds between the FILETIME epoch (1601-01-01) and the Unix epoch (1970-01-01).
const SECONDS_FROM_FILETIME_EPOCH_TO_UNIX_EPOCH: u64 = 11_644_473_600;
const FILETIME_TICKS_PER_SECOND: u64 = 10_000_000;
const NANOSECONDS_PER_FILETIME_TICK: u64 = 100;

/// Converts a FILETIME (100 ns ticks since 1601) into a `SystemTime`.
pub(super) fn filetime_to_system_time(filetime_ticks: i64) -> SystemTime {
    let ticks = filetime_ticks.max(0).cast_unsigned();
    let seconds_since_filetime_epoch = ticks / FILETIME_TICKS_PER_SECOND;
    let remaining_nanoseconds = (ticks % FILETIME_TICKS_PER_SECOND) * NANOSECONDS_PER_FILETIME_TICK;
    let remaining_nanoseconds = u32::try_from(remaining_nanoseconds).unwrap_or(0);

    if seconds_since_filetime_epoch >= SECONDS_FROM_FILETIME_EPOCH_TO_UNIX_EPOCH {
        let since_unix_epoch = Duration::new(
            seconds_since_filetime_epoch - SECONDS_FROM_FILETIME_EPOCH_TO_UNIX_EPOCH,
            remaining_nanoseconds,
        );
        SystemTime::UNIX_EPOCH
            .checked_add(since_unix_epoch)
            .unwrap_or(SystemTime::UNIX_EPOCH)
    } else {
        SystemTime::UNIX_EPOCH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_the_unix_epoch() {
        let unix_epoch_as_filetime = 116_444_736_000_000_000_i64;

        assert_eq!(
            filetime_to_system_time(unix_epoch_as_filetime),
            SystemTime::UNIX_EPOCH
        );
    }

    #[test]
    fn converts_a_later_time_with_sub_second_precision() {
        let one_and_a_half_seconds_after_unix_epoch = 116_444_736_015_000_000_i64;

        assert_eq!(
            filetime_to_system_time(one_and_a_half_seconds_after_unix_epoch),
            SystemTime::UNIX_EPOCH + Duration::from_millis(1500)
        );
    }

    #[test]
    fn clamps_times_before_the_unix_epoch() {
        assert_eq!(filetime_to_system_time(0), SystemTime::UNIX_EPOCH);
    }
}
