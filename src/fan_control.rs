use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanId {
    Cpu,
    Gpu,
}

pub fn set_manual_speed(_fan: FanId, _percent: u8) -> Result<()> {
    Ok(())
}

pub fn set_auto_mode() -> Result<()> {
    Ok(())
}
