/// Direct port of Citan's Moon.java + JulianDay.java to Rust
/// to verify we get the exact same results as the game server

fn main() {
    // Port of JulianDay.java
    fn julian_day(y: i32, mo: i32, day: i32) -> f64 {
        let mut year = y;
        let mut month = mo;
        let d = day as f64; // day fraction = just the integer day (midnight)
        
        match month {
            1 | 2 => { year -= 1; month += 12; }
            _ => {}
        }
        
        let b = if true /* dateIsGregorian - always true for modern dates */ {
            let a = (year as f64 / 100.0).floor() as i32;
            2 - a + (a as f64 / 4.0).floor() as i32
        } else { 0 };
        
        (365.25 * (year + 4716) as f64).floor() 
            + (30.6001 * (month + 1) as f64).floor() 
            + d + b as f64 - 1524.5
    }
    
    // Port of Moon.java
    fn moon_d(t: f64) -> f64 { // Mean elongation of the moon (Meeus 47.2)
        297.8501921 + t * (445267.1114034 + t * (-0.0018819 + t * (1.0 / 545868.0 - t / 113065000.0)))
    }
    fn moon_m(t: f64) -> f64 { // Sun's mean anomaly (Meeus 47.3)
        357.5291092 + t * (35999.0502909 + t * (-0.0001536 + t / 24490000.0))
    }
    fn moon_m_prime(t: f64) -> f64 { // Moon's mean anomaly (Meeus 47.4)
        134.9633964 + t * (477198.8675055 + t * (0.0087414 + t * (1.0 / 69699.0 - t / 14712000.0)))
    }
    fn phase_angle(t: f64) -> f64 { // Meeus 41.1
        let d = moon_d(t);
        let m = moon_m(t);
        let m_prime = moon_m_prime(t);
        180.0
            - d
            - 6.289 * m_prime.to_radians().sin()
            + 2.100 * m.to_radians().sin()
            - 1.274 * (2.0 * d - m_prime).to_radians().sin()
            - 0.658 * (2.0 * d).to_radians().sin()
            - 0.214 * (2.0 * m_prime).to_radians().sin()
            - 0.110 * d.to_radians().sin()
    }
    fn is_waning(i: f64) -> bool {
        let s = i.to_radians().sin();
        if s < 0.0 { return true; }
        if s > 0.0 { return false; }
        i.to_radians().cos() > 0.0
    }
    fn illuminated_fraction(i: f64) -> f64 {
        (1.0 + i.to_radians().cos()) * 0.5
    }
    
    println!("=== Direct port of server's Moon.java + JulianDay.java ===");
    println!("Apr  Day | JD          | t           | phase_angle | illum  | waning");
    
    for day in 6..=14 {
        let jd = julian_day(2026, 4, day);
        let t = (jd - 2451545.0) / 36525.0; // Meeus 22.1
        let i = phase_angle(t);
        let illum = illuminated_fraction(i);
        let wan = is_waning(i);
        let marker = if day == 9 { " <-- TODAY" } else { "" };
        println!("Apr {:2}   | {:.4} | {:.10} | {:.4}     | {:.4} | {:5}{}", 
            day, jd, t, i, illum, wan, marker);
    }
    
    // Now run PgMoon-style algorithm with this exact server math
    println!("\n=== Server-algorithm phase assignment ===");
    
    struct DayInfo {
        month: u32,
        day: i32,
        illum: f64,
        waning: bool,
    }
    
    let mut days: Vec<DayInfo> = Vec::new();
    for offset in -5i32..=35 {
        let apr_day = 9 + offset;
        let (m, d) = if apr_day < 1 { (3i32, 31 + apr_day) }
            else if apr_day > 30 { (5, apr_day - 30) }
            else { (4, apr_day) };
        let jd = julian_day(2026, m, d);
        let t = (jd - 2451545.0) / 36525.0;
        let i = phase_angle(t);
        days.push(DayInfo { 
            month: m as u32, day: d, 
            illum: illuminated_fraction(i), 
            waning: is_waning(i) 
        });
    }
    
    let mut assigned: Vec<Option<&str>> = vec![None; days.len()];
    
    // Full Moon
    for i in 1..days.len() {
        if !days[i-1].waning && days[i].waning {
            if i >= 1 { assigned[i-1] = Some("Full Moon"); }
            assigned[i] = Some("Full Moon");
            if i + 1 < days.len() { assigned[i+1] = Some("Full Moon"); }
            break;
        }
    }
    // New Moon
    for i in 1..days.len() {
        if days[i-1].waning && !days[i].waning {
            if i >= 1 { assigned[i-1] = Some("New Moon"); }
            assigned[i] = Some("New Moon");
            if i + 1 < days.len() { assigned[i+1] = Some("New Moon"); }
            break;
        }
    }
    // First Quarter
    for i in 1..days.len() {
        if !days[i].waning && !days[i-1].waning && days[i-1].illum <= 0.5 && days[i].illum > 0.5 {
            if i >= 1 { assigned[i-1] = Some("First Quarter"); }
            assigned[i] = Some("First Quarter");
            if i + 1 < days.len() { assigned[i+1] = Some("First Quarter"); }
            break;
        }
    }
    // Last Quarter
    for i in 1..days.len() {
        if days[i].waning && days[i-1].waning && days[i-1].illum >= 0.5 && days[i].illum < 0.5 {
            if i >= 1 { assigned[i-1] = Some("Last Quarter"); }
            assigned[i] = Some("Last Quarter");
            if i + 1 < days.len() { assigned[i+1] = Some("Last Quarter"); }
            break;
        }
    }
    // Fill remaining
    for i in 0..days.len() {
        if assigned[i].is_none() {
            assigned[i] = Some(if days[i].waning {
                if days[i].illum >= 0.5 { "Waning Gibbous" } else { "Waning Crescent" }
            } else {
                if days[i].illum <= 0.5 { "Waxing Crescent" } else { "Waxing Gibbous" }
            });
        }
    }
    
    for i in 0..days.len() {
        let d = &days[i];
        let marker = if d.month == 4 && d.day == 9 { " <-- TODAY" } else { "" };
        println!("{:2}/{:2}: illum={:.6} wan={:5} -> {:16}{}", 
            d.month, d.day, d.illum, d.waning, assigned[i].unwrap(), marker);
    }
}
