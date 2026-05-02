use derive_more::{Add, AddAssign, Sub, SubAssign};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(
    Default,
    PartialEq,
    PartialOrd,
    Debug,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    Serialize,
    Deserialize,
    Clone,
    Copy,
)]
/// A single denomination of in-game currency backed by an `f64` amount.
pub struct Currency(pub f64);

impl Currency {
    /// Subtracts `other` from `self`, clamping the result to zero if it would go negative.
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

#[derive(Default, PartialEq, Debug, Serialize, Deserialize, Clone, Add, AddAssign)]
/// A bundle of three resource denominations: bronze, silver, and gold.
pub struct Resources {
    pub bronze: Currency,
    pub silver: Currency,
    pub gold: Currency,
}

impl Resources {
    /// Creates a `Resources` with explicit amounts for each denomination.
    pub const fn new(bronze: f64, silver: f64, gold: f64) -> Self {
        Resources {
            bronze: Currency(bronze),
            silver: Currency(silver),
            gold: Currency(gold),
        }
    }

    /// Returns a `Resources` with all denominations set to zero.
    pub const fn zero() -> Self {
        Resources::new(0.0, 0.0, 0.0)
    }

    /// Creates a `Resources` with only the bronze denomination set.
    pub const fn from_bronze(bronze: f64) -> Self {
        Resources::new(bronze, 0.0, 0.0)
    }

    /// Creates a `Resources` with only the silver denomination set.
    pub const fn from_silver(silver: f64) -> Self {
        Resources::new(0.0, silver, 0.0)
    }

    /// Creates a `Resources` with only the gold denomination set.
    pub const fn from_gold(gold: f64) -> Self {
        Resources::new(0.0, 0.0, gold)
    }

    /// Returns a single-line display of all non-zero denominations.
    pub const fn fmt_oneline(&self) -> impl Display {
        ResourcesFmtOneline { parent: self }
    }

    /// Returns a multi-line display with each non-zero denomination on its own line.
    pub const fn fmt_multiline(&self) -> impl Display {
        ResourcesFmtMultiline { parent: self }
    }

    /// Subtracts `other` from `self` per denomination, clamping each to zero.
    pub fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            bronze: self.bronze.saturating_sub(other.bronze),
            silver: self.silver.saturating_sub(other.silver),
            gold: self.gold.saturating_sub(other.gold),
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
