pub const fn chrono_offset_date_time_to_prost_timestamp(
    time: time::OffsetDateTime,
) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: time.unix_timestamp(),
        nanos: time.nanosecond() as i32,
    }
}
