use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Add;

#[derive(
    Default,
    PartialEq,
    PartialOrd,
    Debug,
    derive_more::derive::Add,
    derive_more::derive::Sub,
    derive_more::derive::AddAssign,
    derive_more::derive::SubAssign,
    Serialize,
    Deserialize,
    Clone,
    Copy,
)]
pub struct Currency(pub f64);

impl Currency {
    pub fn saturating_sub(&self, other: Self) -> Self {
        let delta = self.0 - other.0;
        Currency(if delta < 0.0 { 0.0 } else { delta })
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = self.0;
        if value.abs() < 1000.0 {
            write!(f, "{}", value as i64)
        } else {
            let exp = (value.abs().log10().floor() as i32) / 3 * 3;
            let mantissa = value / 10f64.powi(exp);
            write!(f, "{:.1}e{}", mantissa, exp)
        }
    }
}

#[derive(Default, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Resources {
    pub bronze: Currency,
    pub silver: Currency,
    pub gold: Currency,
}

impl Resources {
    pub const fn new(bronze: f64, silver: f64, gold: f64) -> Self {
        Resources {
            bronze: Currency(bronze),
            silver: Currency(silver),
            gold: Currency(gold),
        }
    }

    pub const fn zero() -> Self {
        Resources::new(0.0, 0.0, 0.0)
    }

    pub const fn from_bronze(bronze: f64) -> Self {
        Resources::new(bronze, 0.0, 0.0)
    }

    pub const fn from_silver(silver: f64) -> Self {
        Resources::new(0.0, silver, 0.0)
    }

    pub const fn from_gold(gold: f64) -> Self {
        Resources::new(0.0, 0.0, gold)
    }

    pub const fn fmt_oneline(&self) -> impl Display {
        ResourcesFmtOneline { parent: self }
    }

    pub const fn fmt_multiline(&self) -> impl Display {
        ResourcesFmtMultiline { parent: self }
    }

    pub fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            bronze: self.bronze.saturating_sub(other.bronze),
            silver: self.silver.saturating_sub(other.silver),
            gold: self.gold.saturating_sub(other.gold),
        }
    }
}

impl Add for &Resources {
    type Output = Resources;
    fn add(self, rhs: &Resources) -> Resources {
        Resources {
            bronze: self.bronze + rhs.bronze,
            silver: self.silver + rhs.silver,
            gold: self.gold + rhs.gold,
        }
    }
}

impl PartialOrd for Resources {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let bronze_le = self.bronze <= other.bronze;
        let silver_le = self.silver <= other.silver;
        let gold_le = self.gold <= other.gold;
        let bronze_ge = self.bronze >= other.bronze;
        let silver_ge = self.silver >= other.silver;
        let gold_ge = self.gold >= other.gold;
        match (
            bronze_le && silver_le && gold_le,
            bronze_ge && silver_ge && gold_ge,
        ) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            (false, false) => None,
        }
    }
}

const BRONZE_SYMBOL: char = '🟤';
const SILVER_SYMBOL: char = '⚪';
const GOLD_SYMBOL: char = '🟡';

struct ResourcesFmtOneline<'a> {
    parent: &'a Resources,
}

impl Display for ResourcesFmtOneline<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let write_gold = self.parent.gold.0 != 0.0;
        let write_silver = self.parent.silver.0 != 0.0 || write_gold;
        if write_gold {
            write!(f, "{} {}", self.parent.gold, GOLD_SYMBOL)?;
        }
        if write_silver {
            write!(f, "{} {}", self.parent.silver, SILVER_SYMBOL)?;
        }
        write!(f, "{} {}", self.parent.bronze, BRONZE_SYMBOL)
    }
}

struct ResourcesFmtMultiline<'a> {
    parent: &'a Resources,
}

impl Display for ResourcesFmtMultiline<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let write_gold = self.parent.gold.0 != 0.0;
        let write_silver = self.parent.silver.0 != 0.0 || write_gold;
        if write_gold {
            writeln!(f, "{} {}", self.parent.gold, GOLD_SYMBOL)?;
        }
        if write_silver {
            writeln!(f, "{} {}", self.parent.silver, SILVER_SYMBOL)?;
        }
        writeln!(f, "{} {}", self.parent.bronze, BRONZE_SYMBOL)
    }
}
