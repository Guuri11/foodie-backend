use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    New,
    Opened,
    AlmostEmpty,
    Finished,
}

impl std::fmt::Display for ProductStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductStatus::New => write!(f, "new"),
            ProductStatus::Opened => write!(f, "opened"),
            ProductStatus::AlmostEmpty => write!(f, "almost_empty"),
            ProductStatus::Finished => write!(f, "finished"),
        }
    }
}

impl std::str::FromStr for ProductStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "new" => Ok(ProductStatus::New),
            "opened" => Ok(ProductStatus::Opened),
            "almost_empty" => Ok(ProductStatus::AlmostEmpty),
            "finished" => Ok(ProductStatus::Finished),
            _ => Err(format!("Invalid product status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductLocation {
    Fridge,
    Pantry,
    Freezer,
}

impl std::fmt::Display for ProductLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductLocation::Fridge => write!(f, "fridge"),
            ProductLocation::Pantry => write!(f, "pantry"),
            ProductLocation::Freezer => write!(f, "freezer"),
        }
    }
}

impl std::str::FromStr for ProductLocation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fridge" => Ok(ProductLocation::Fridge),
            "pantry" => Ok(ProductLocation::Pantry),
            "freezer" => Ok(ProductLocation::Freezer),
            _ => Err(format!("Invalid product location: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductOutcome {
    Used,
    ThrownAway,
}

impl std::fmt::Display for ProductOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductOutcome::Used => write!(f, "used"),
            ProductOutcome::ThrownAway => write!(f, "thrown_away"),
        }
    }
}

impl std::str::FromStr for ProductOutcome {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "used" => Ok(ProductOutcome::Used),
            "thrown_away" => Ok(ProductOutcome::ThrownAway),
            _ => Err(format!("Invalid product outcome: {}", s)),
        }
    }
}
