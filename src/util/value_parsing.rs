use thiserror::Error;

pub fn f32_gte_0(s: &str) -> Result<f32, anyhow::Error> {
    let x = s.parse()?;
    if x >= 0. {
        Ok(x)
    } else {
        Err(Error::LessThanZero.into())
    }
}

pub fn usize_gte_1(s: &str) -> Result<usize, anyhow::Error> {
    let x = s.parse()?;
    if x >= 1 {
        Ok(x)
    } else {
        Err(Error::LessThanOne.into())
    }
}

#[derive(Clone, Copy, Debug, Error)]
pub enum Error {
    #[error("Must be >= 0")]
    LessThanZero,
    #[error("Must be >= 1")]
    LessThanOne,
}
