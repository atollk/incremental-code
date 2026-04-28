use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Debug, Serialize, Deserialize)]
pub struct Resources {
    bronze: f64,
    silver: f64,
    gold: f64,
}

impl Resources {
    pub const fn from_bronze(bronze: f64) -> Self {
        Resources {
            bronze,
            silver: 0.0,
            gold: 0.0,
        }
    }

    pub const fn from_silver(silver: f64) -> Self {
        Resources {
            bronze: 0.0,
            silver,
            gold: 0.0,
        }
    }

    pub const fn from_gold(gold: f64) -> Self {
        Resources {
            bronze: 0.0,
            silver: 0.0,
            gold,
        }
    }
}
