use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Default, PartialEq, Debug, Serialize, Deserialize)]
pub struct Resources {
    bronze: Currency,
    silver: Currency,
    gold: Currency,
}

#[derive(Default, PartialEq, Debug, Serialize, Deserialize)]
pub struct Currency(f64);

impl fmt::Display for Currency {
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

impl Resources {
    pub const fn from_bronze(bronze: f64) -> Self {
        Resources {
            bronze: Currency(bronze),
            silver: Currency(0.0),
            gold: Currency(0.0),
        }
    }

    pub const fn from_silver(silver: f64) -> Self {
        Resources {
            bronze: Currency(0.0),
            silver: Currency(silver),
            gold: Currency(0.0),
        }
    }

    pub const fn from_gold(gold: f64) -> Self {
        Resources {
            bronze: Currency(0.0),
            silver: Currency(0.0),
            gold: Currency(gold),
        }
    }
}

const BRONZE_SYMBOL: char = '🟤';
const SILVER_SYMBOL: char = '⚪';
const GOLD_SYMBOL: char = '🟡';
const DIAMOND_SYMBOL: char = '💎';

impl Display for Resources {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let write_gold = self.gold.0 != 0.0;
        let write_silver = self.silver.0 != 0.0 || write_gold;
        if write_gold {
            write!(f, "{} {}", self.gold, GOLD_SYMBOL)?;
        }
        if write_silver {
            write!(f, "{} {}", self.silver, SILVER_SYMBOL)?;
        }
        write!(f, "{} {}", self.bronze, BRONZE_SYMBOL)
    }
}
