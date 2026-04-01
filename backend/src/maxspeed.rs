use utils::Tags;

/// Effektiven Maxspeed aus OSM-Tags ermitteln.
/// - `maxspeed` direkt → zurückgeben
/// - `maxspeed:forward` + `maxspeed:backward` → höheren Original-String zurückgeben
/// - Alles andere → None (keine Maxspeed-Info vorhanden)
pub fn get_maxspeed_from_tags(tags: &Tags) -> Option<String> {
    if let Some(v) = tags.get("maxspeed") {
        return Some(v.clone());
    }
    match (tags.get("maxspeed:forward"), tags.get("maxspeed:backward")) {
        (Some(fwd), Some(bwd)) => pick_higher_maxspeed(&fwd, &bwd),
        _ => None,
    }
}

/// Numerischen Teil eines Maxspeed-Wertes in km/h-Äquivalent parsen.
/// Unterstützt Format "50" oder "30 mph" (Zahl, Optional-Leerzeichen, Optional-Einheit).
fn parse_maxspeed_value(s: &str) -> Option<f64> {
    let mut parts = s.splitn(2, ' ');
    let num: f64 = parts.next()?.trim().parse().ok()?;
    match parts.next().map(|u| u.trim()) {
        Some("mph") => Some(num * 1.60934),
        Some("knots") => Some(num * 1.852),
        _ => Some(num),
    }
}

/// Den String-Wert der höheren Maxspeed zurückgeben (Original, nicht konvertiert).
fn pick_higher_maxspeed(a: &str, b: &str) -> Option<String> {
    let a_val = parse_maxspeed_value(a)?;
    let b_val = parse_maxspeed_value(b)?;
    if a_val >= b_val {
        Some(a.to_string())
    } else {
        Some(b.to_string())
    }
}
