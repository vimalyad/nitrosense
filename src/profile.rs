use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerProfile {
    pub name: String,
}

pub fn read_active_profile() -> Result<Option<PowerProfile>> {
    Ok(None)
}

pub fn read_profile_choices() -> Result<Vec<PowerProfile>> {
    Ok(Vec::new())
}
