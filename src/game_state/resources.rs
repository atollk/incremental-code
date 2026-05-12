use derive_more::{Add, AddAssign, Sub, SubAssign};
use itertools::Itertools;
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
pub struct Resources {
    pub bronze: Currency,
    pub silver: Currency,
    pub gold: Currency,
    pub diamond: Currency,
    pub stars: Currency,
}

impl Resources {
    /// Creates a `Resources` with explicit amounts for each denomination.
    pub const fn new(bronze: f64, silver: f64, gold: f64, diamond: f64, stars: f64) -> Self {
        Resources {
            bronze: Currency(bronze),
            silver: Currency(silver),
            gold: Currency(gold),
            diamond: Currency(diamond),
            stars: Currency(stars),
        }
    }

    /// Returns a `Resources` with all denominations set to zero.
    pub const fn zero() -> Self {
        Resources::new(0.0, 0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a `Resources` with only the bronze denomination set.
    pub const fn from_bronze(bronze: f64) -> Self {
        Resources::new(bronze, 0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a `Resources` with only the silver denomination set.
    pub const fn from_silver(silver: f64) -> Self {
        Resources::new(0.0, silver, 0.0, 0.0, 0.0)
    }

    /// Creates a `Resources` with only the gold denomination set.
    pub const fn from_gold(gold: f64) -> Self {
        Resources::new(0.0, 0.0, gold, 0.0, 0.0)
    }

    pub const fn from_diamond(diamond: f64) -> Self {
        Resources::new(0.0, 0.0, 0.0, diamond, 0.0)
    }

    /// Returns a single-line display of all non-zero denominations.
    pub const fn fmt_oneline(&self) -> impl Display {
        ResourcesFmt {
            parent: self,
            separator: " ",
        }
    }

    /// Returns a multi-line display with each non-zero denomination on its own line.
    pub const fn fmt_multiline(&self) -> impl Display {
        ResourcesFmt {
            parent: self,
            separator: "\n",
        }
    }

    /// Subtracts `other` from `self` per denomination, clamping each to zero.
    pub fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            bronze: self.bronze.saturating_sub(other.bronze),
            silver: self.silver.saturating_sub(other.silver),
            gold: self.gold.saturating_sub(other.gold),
            diamond: self.diamond.saturating_sub(other.diamond),
            stars: self.stars.saturating_sub(other.stars),
        }
    }
}

impl PartialOrd for Resources {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let bronze_le = self.bronze <= other.bronze;
        let silver_le = self.silver <= other.silver;
        let gold_le = self.gold <= other.gold;
        let diamond_le = self.diamond <= other.diamond;
        let stars_le = self.stars <= other.stars;
        let bronze_ge = self.bronze >= other.bronze;
        let silver_ge = self.silver >= other.silver;
        let gold_ge = self.gold >= other.gold;
        let diamond_ge = self.diamond >= other.diamond;
        let stars_ge = self.stars >= other.stars;
        match (
            bronze_le && silver_le && gold_le && diamond_le && stars_le,
            bronze_ge && silver_ge && gold_ge && diamond_ge && stars_ge,
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
const DIAMOND_SYMBOL: char = '💎';
const STAR_SYMBOL: char = '⭐';

struct ResourcesFmt<'a> {
    parent: &'a Resources,
    separator: &'a str,
}

impl Display for ResourcesFmt<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let write_stars = self.parent.stars.0 != 0.0;
        let write_diamond = self.parent.stars.0 != 0.0 || write_stars;
        let write_gold = self.parent.gold.0 != 0.0 || write_diamond;
        let write_silver = self.parent.silver.0 != 0.0 || write_gold;
        let write_strings = [
            (write_stars, self.parent.stars, STAR_SYMBOL),
            (write_diamond, self.parent.diamond, DIAMOND_SYMBOL),
            (write_gold, self.parent.gold, GOLD_SYMBOL),
            (write_silver, self.parent.silver, SILVER_SYMBOL),
            (true, self.parent.bronze, BRONZE_SYMBOL),
        ]
        .into_iter()
        .filter_map(|(write, currency, symbol)| {
            if write {
                Some(format!("{currency} {symbol}"))
            } else {
                None
            }
        })
        .intersperse(self.separator.to_string());
        for s in write_strings {
            write!(f, "{}", s)?;
        }
        Ok(())
    }
}
