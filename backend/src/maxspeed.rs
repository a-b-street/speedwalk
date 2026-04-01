use utils::Tags;

/// Determine effective maxspeed from OSM tags.
/// - `maxspeed` directly → return it
/// - `maxspeed:forward` + `maxspeed:backward` → return the higher original string
/// - Anything else → None (no maxspeed info available)
pub fn get_maxspeed_from_tags(tags: &Tags) -> Option<String> {
    if let Some(v) = tags.get("maxspeed") {
        return Some(v.clone());
    }
    match (tags.get("maxspeed:forward"), tags.get("maxspeed:backward")) {
        (Some(fwd), Some(bwd)) => pick_higher_maxspeed(&fwd, &bwd),
        _ => None,
    }
}

/// Parse the numeric part of a maxspeed value into a km/h equivalent.
/// Supports formats like "50" or "30 mph" (number, optional space, optional unit).
fn parse_maxspeed_value(s: &str) -> Option<f64> {
    let mut parts = s.splitn(2, ' ');
    let num: f64 = parts.next()?.trim().parse().ok()?;
    match parts.next().map(|u| u.trim()) {
        Some("mph") => Some(num * 1.60934),
        Some("knots") => Some(num * 1.852),
        _ => Some(num),
    }
}

/// Return the string value of the higher maxspeed (original, not converted).
fn pick_higher_maxspeed(a: &str, b: &str) -> Option<String> {
    let a_val = parse_maxspeed_value(a)?;
    let b_val = parse_maxspeed_value(b)?;
    if a_val >= b_val {
        Some(a.to_string())
    } else {
        Some(b.to_string())
    }
}
