//! Issue #13 vehicle score taxonomy and weight rules.

use parser_contract::events::VehicleScoreCategory;

/// Maps raw OCAP vehicle class evidence to an issue #13 vehicle score category.
#[must_use]
pub fn category_from_vehicle_class(raw_class: Option<&str>) -> VehicleScoreCategory {
    let Some(normalized) =
        raw_class.map(str::trim).filter(|value| !value.is_empty()).map(str::to_ascii_lowercase)
    else {
        return VehicleScoreCategory::Unknown;
    };

    if normalized == "static-weapon"
        || normalized.contains("static")
        || normalized.contains("weapon")
        || normalized.contains("mortar")
    {
        return VehicleScoreCategory::StaticWeapon;
    }
    if normalized == "apc"
        || normalized.contains("apc")
        || normalized.contains("btr")
        || normalized.contains("bmp")
        || normalized.contains("brdm")
        || normalized.contains("m113")
        || normalized.contains("stryker")
    {
        return VehicleScoreCategory::Apc;
    }
    if normalized == "tank"
        || normalized.contains("tank")
        || normalized.contains("t72")
        || normalized.contains("t90")
        || normalized.contains("m1a")
        || normalized.contains("abrams")
    {
        return VehicleScoreCategory::Tank;
    }
    if normalized == "heli"
        || normalized.contains("heli")
        || normalized.contains("ah1")
        || normalized.contains("ah64")
        || normalized.contains("mi24")
        || normalized.contains("mi8")
        || normalized.contains("uh60")
    {
        return VehicleScoreCategory::Heli;
    }
    if normalized == "plane"
        || normalized.contains("plane")
        || normalized.contains("jet")
        || normalized.contains("su25")
        || normalized.contains("a10")
    {
        return VehicleScoreCategory::Plane;
    }
    if normalized == "truck"
        || normalized.contains("truck")
        || normalized.contains("ural")
        || normalized.contains("kamaz")
        || normalized.contains("mtvr")
        || normalized.contains("hemtt")
    {
        return VehicleScoreCategory::Truck;
    }
    if normalized == "car"
        || normalized.contains("car")
        || normalized.contains("offroad")
        || normalized.contains("hmmwv")
        || normalized.contains("uaz")
        || normalized.contains("gaz")
    {
        return VehicleScoreCategory::Car;
    }

    VehicleScoreCategory::Unknown
}

/// Returns the issue #13 matrix weight for an attacker/target category pair.
#[must_use]
#[allow(
    clippy::match_same_arms,
    reason = "the match mirrors the issue #13 matrix rows for auditability"
)]
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
pub const fn teamkill_penalty_weight(raw_weight: f64) -> f64 {
    if raw_weight < 1.0 { 1.0 } else { raw_weight }
}
