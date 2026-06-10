pub fn next_backoff_ms(attempt: u32, min_delay_ms: u64, max_delay_ms: u64) -> u64 {
    let exponent = 2_u64.saturating_pow(attempt.min(10));
    (min_delay_ms.saturating_mul(exponent)).min(max_delay_ms)
}

