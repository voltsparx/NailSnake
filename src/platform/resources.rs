use anyhow::{bail, Context, Result};

const MIN_MEMORY_RESERVATION_BYTES: usize = 8 * 1024 * 1024;

pub fn ensure_hardware_resources() -> Result<()> {
    let cpu_threads = std::thread::available_parallelism()
        .context("could not inspect available CPU parallelism")?
        .get();

    if cpu_threads == 0 {
        bail!("not enough CPU resources available to run NailSnake safely");
    }

    let mut memory_probe = Vec::<u8>::new();
    memory_probe
        .try_reserve_exact(MIN_MEMORY_RESERVATION_BYTES)
        .context("not enough available memory to run NailSnake safely")?;

    Ok(())
}
