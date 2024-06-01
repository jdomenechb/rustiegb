use std::time::Instant;

#[allow(dead_code)]
pub fn bench<F>(to_bench: F, label: &str)
where
    F: Fn(),
{
    let start_time = Instant::now();

    to_bench();

    let end_time = Instant::now();
    let elapsed_time = end_time - start_time;

    println!(
        "[BENCH][{}] Time elapsed {} s {} ms {} ns",
        label,
        elapsed_time.as_secs(),
        elapsed_time.subsec_millis(),
        elapsed_time.subsec_nanos()
    );
}
