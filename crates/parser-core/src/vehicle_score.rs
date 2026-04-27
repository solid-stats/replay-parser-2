//! Issue #13 vehicle score taxonomy and weight rules.

use parser_contract::events::VehicleScoreCategory;

/// Maps raw OCAP vehicle class evidence to an issue #13 vehicle score category.
#[must_use]
pub fn category_from_vehicle_class(raw_class: Option<&str>) -> VehicleScoreCategory {
    match raw_class {
        Some("static-weapon") => VehicleScoreCategory::StaticWeapon,
        Some("car") => VehicleScoreCategory::Car,
        Some("truck") => VehicleScoreCategory::Truck,
        Some("apc") => VehicleScoreCategory::Apc,
        Some("tank") => VehicleScoreCategory::Tank,
        Some("heli") => VehicleScoreCategory::Heli,
        Some("plane") => VehicleScoreCategory::Plane,
        Some(_) | None => VehicleScoreCategory::Unknown,
    }
}

/// Returns the issue #13 matrix weight for an attacker/target category pair.
#[must_use]
pub const fn vehicle_score_weight(
    attacker: VehicleScoreCategory,
    target: VehicleScoreCategory,
) -> Option<f64> {
    use VehicleScoreCategory::{Apc, Car, Heli, Plane, Player, StaticWeapon, Tank, Truck, Unknown};

    match (attacker, target) {
        (Unknown, _) | (_, Unknown) => None,
        (StaticWeapon | Car | Truck, StaticWeapon | Car | Truck | Apc) => Some(1.0),
        (StaticWeapon | Car | Truck, Tank) => Some(1.5),
        (StaticWeapon | Car | Truck, Heli | Plane | Player) => Some(2.0),
        (Apc, StaticWeapon) => Some(0.5),
        (Apc, Car | Truck | Apc | Tank) => Some(1.0),
        (Apc, Heli | Plane | Player) => Some(2.0),
        (Tank, StaticWeapon) => Some(0.25),
        (Tank, Car | Truck | Apc) => Some(0.5),
        (Tank, Tank) => Some(1.0),
        (Tank, Heli) => Some(1.5),
        (Tank, Plane | Player) => Some(2.0),
        (Heli, StaticWeapon | Car) => Some(0.5),
        (Heli, Truck | Apc) => Some(1.0),
        (Heli, Tank | Heli) => Some(1.5),
        (Heli, Plane | Player) => Some(2.0),
        (Plane, StaticWeapon) => Some(0.25),
        (Plane, Car | Truck | Apc) => Some(0.5),
        (Plane, Tank) => Some(1.0),
        (Plane, Heli) => Some(1.5),
        (Plane, Plane | Player) => Some(2.0),
        (Player, _) => None,
    }
}

/// Applies the issue #13 teamkill penalty clamp.
#[must_use]
pub fn teamkill_penalty_weight(raw_weight: f64) -> f64 {
    raw_weight.max(1.0)
}
