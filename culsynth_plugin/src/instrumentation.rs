#![allow(dead_code)]

#[cfg(feature = "instrumentation")]
static PROCESS_NANOS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[cfg(feature = "instrumentation")]
pub struct Instrumentation(std::time::Instant);

#[cfg(not(feature = "instrumentation"))]
pub struct Instrumentation();

pub fn begin() -> Instrumentation {
    #[cfg(feature = "instrumentation")]
    {
        Instrumentation(std::time::Instant::now())
    }
    #[cfg(not(feature = "instrumentation"))]
    {
        Instrumentation()
    }
}

pub fn end(_i: Instrumentation) {
    #[cfg(feature = "instrumentation")]
    {
        let duration = _i.0.elapsed();
        PROCESS_NANOS.store(
            duration.subsec_nanos(),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}

pub fn get_last() -> u32 {
    #[cfg(feature = "instrumentation")]
    {
        PROCESS_NANOS.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(feature = "instrumentation"))]
    {
        0
    }
}
