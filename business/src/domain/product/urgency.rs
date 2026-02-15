use chrono::Utc;

use super::model::Product;

/// Urgency levels for product expiry.
#[derive(Debug, Clone, PartialEq)]
pub enum UrgencyLevel {
    /// Product is fresh, no urgency.
    Ok,
    /// Product expires in 1-2 days.
    UseSoon,
    /// Product expires today.
    UseToday,
    /// Product has expired.
    WouldntTrust,
}

impl std::fmt::Display for UrgencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrgencyLevel::Ok => write!(f, "ok"),
            UrgencyLevel::UseSoon => write!(f, "use_soon"),
            UrgencyLevel::UseToday => write!(f, "use_today"),
            UrgencyLevel::WouldntTrust => write!(f, "wouldnt_trust"),
        }
    }
}

const EXPIRING_SOON_DAYS: i64 = 2;

/// Calculates the number of days until a product expires.
///
/// Returns `None` if the product has no expiry date.
/// Returns 0 for products expiring today, negative for expired products.
pub fn days_until_expiry(product: &Product) -> Option<i64> {
    let date = product.expiry_date.or(product.estimated_expiry_date)?;

    let today = Utc::now().date_naive();
    let expiry_day = date.date_naive();

    Some((expiry_day - today).num_days())
}

/// Determines the urgency level of a product.
///
/// Business rules:
/// - Expired -> WouldntTrust
/// - Expires today (0 days) -> UseToday
/// - Expires in 1-2 days -> UseSoon
/// - Expires in 3+ days or no date -> Ok
pub fn get_urgency_level(product: &Product) -> UrgencyLevel {
    let date = product.expiry_date.or(product.estimated_expiry_date);
    if date.is_none() {
        return UrgencyLevel::Ok;
    }

    if is_expired(product) {
        return UrgencyLevel::WouldntTrust;
    }

    let days = match days_until_expiry(product) {
        Some(d) => d,
        None => return UrgencyLevel::Ok,
    };

    if days == 0 {
        return UrgencyLevel::UseToday;
    }

    if is_expiring_soon(product) {
        return UrgencyLevel::UseSoon;
    }

    UrgencyLevel::Ok
}

/// Returns true if the product is expired.
pub fn is_expired(product: &Product) -> bool {
    let date = product.expiry_date.or(product.estimated_expiry_date);
    match date {
        Some(d) => d < Utc::now(),
        None => false,
    }
}

/// Returns true if the product is expiring soon (within 2 days, not expired).
pub fn is_expiring_soon(product: &Product) -> bool {
    match days_until_expiry(product) {
        Some(days) => (0..=EXPIRING_SOON_DAYS).contains(&days),
        None => false,
    }
}
