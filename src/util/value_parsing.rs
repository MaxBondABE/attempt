use std::{num::ParseFloatError, str::FromStr};

pub fn f32_gte_0(s: &str) -> Result<f32, anyhow::Error> {
    let x = s.parse()?;
    if x >= 0. {
        Ok(x)
    } else {
        Err(ValueError::LessThanZero.into())
    }
}

pub fn usize_gte_1(s: &str) -> Result<usize, anyhow::Error> {
    let x = s.parse()?;
    if x >= 1 {
        Ok(x)
    } else {
        Err(ValueError::LessThanOne.into())
    }
}

pub fn time_duration(s: &str) -> Result<f32, anyhow::Error> {
    let mut interval = 0.;
    let mut remaining = s.trim();
    if remaining.is_empty() {
        return Err(ValueError::EmptyTimeString.into());
    }

    if s.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Ok(s.parse()?);
    }

    while !remaining.is_empty() {
        let number_idx = remaining
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .map(|c| c.len_utf8())
            .sum();
        let (n, r) = remaining.split_at(number_idx);
        remaining = r.trim_start();

        let unit_idx = remaining
            .chars()
            .take_while(|c| !(c.is_ascii_digit() || *c == '.' || c.is_whitespace()))
            .map(|c| c.len_utf8())
            .sum::<usize>();
        let (unit, r) = remaining.split_at(unit_idx);
        remaining = r.trim_start();

        if unit.is_empty() {
            return Err(ValueError::InconsistentUnits.into());
        }

        let n: f32 = n.parse()?;
        let unit: TimeUnit = unit.parse()?;
        interval += n * unit.scale();
    }

    Ok(interval)
}

enum TimeUnit {
    Hour,
    Minute,
    Second,
    Millisecond,
    Nanosecond,
}
impl TimeUnit {
    pub fn scale(&self) -> f32 {
        match self {
            TimeUnit::Hour => 3600.,
            TimeUnit::Minute => 60.,
            TimeUnit::Second => 1.,
            TimeUnit::Millisecond => 1e-3,
            TimeUnit::Nanosecond => 1e-6,
        }
    }
}
impl FromStr for TimeUnit {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "h" | "hr" => Ok(Self::Hour),
            "m" | "min" => Ok(Self::Minute),
            "s" => Ok(Self::Second),
            "ms" => Ok(Self::Millisecond),
            "ns" => Ok(Self::Nanosecond),
            _ => Err(ValueError::UnknownTimeUnit),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ValueError {
    #[error("Must be >= 0")]
    LessThanZero,
    #[error("Must be >= 1")]
    LessThanOne,
    #[error("Unknown unit of time")]
    UnknownTimeUnit,
    #[error("Empty time string")]
    EmptyTimeString,
    #[error("Invalid number: {0}")]
    InvalidNumber(#[from] ParseFloatError),
    #[error("If any time value has a unit, all must have units")]
    InconsistentUnits,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_units() {
        assert_eq!(time_duration("5").unwrap(), 5.0);
        assert_eq!(time_duration("5s").unwrap(), 5.0);
        assert_eq!(time_duration("5m").unwrap(), 300.0);
        assert_eq!(time_duration("5min").unwrap(), 300.0);
        assert_eq!(time_duration("1h").unwrap(), 3600.0);
        assert_eq!(time_duration("1hr").unwrap(), 3600.0);
        assert_eq!(time_duration("500ms").unwrap(), 0.5);
        assert_eq!(time_duration("1000ns").unwrap(), 0.001);
    }

    #[test]
    fn multiple_components() {
        assert_eq!(time_duration("1h30m").unwrap(), 3600. + 1800.);
        assert_eq!(time_duration("1h30m10s").unwrap(), 3600. + 1800. + 10.);

        assert_eq!(time_duration("1h 30m").unwrap(), 3600. + 1800.);
        assert_eq!(time_duration("1hr 30min 10s").unwrap(), 3600. + 1800. + 10.);
        assert_eq!(time_duration("  1h  30m  ").unwrap(), 3600. + 1800.)
    }

    #[test]
    fn multiple_components_require_units() {
        assert!(time_duration("10 20 30").is_err());
        assert!(time_duration("1h 30").is_err());
        assert!(time_duration("5 10m").is_err());
        assert!(time_duration("1h 2m 30").is_err());
    }

    #[test]
    fn empty_strings() {
        assert!(time_duration("").is_err());
        assert!(time_duration("   ").is_err());
    }

    #[test]
    fn invalid_numbers() {
        assert!(time_duration("abch").is_err());
        assert!(time_duration("1h abc").is_err());
        assert!(time_duration("1.xyz.222h").is_err());
    }

    #[test]
    fn unknown_units() {
        assert!(time_duration("2y").is_err());
        assert!(time_duration("1h2y").is_err());
        assert!(time_duration("1h 2y").is_err());
    }

    #[test]
    fn components_with_fractional_parts() {
        assert_eq!(time_duration("1.5h").unwrap(), 3600. + 1800.);
        assert_eq!(time_duration("1.5h 30m").unwrap(), 3600. + 1800. + 1800.);
        assert_eq!(time_duration("1.0").unwrap(), 1.);
    }
}
