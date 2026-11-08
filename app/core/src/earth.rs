//! Orientation of the Earth: Delta T, precession, nutation and sidereal time.
//!
//! Precession is the long-term model of Vondrák, Capitaine & Wallace 2011, nutation IAU 2000B and
//! sidereal time follows IAU 2006. Coefficients come from the papers via ERFA, see NOTICE.md.
use crate::deltat_table::{DELTA_T, FIRST_YEAR};
use crate::nut_table::NUTATION;
use std::f64::consts::TAU;

pub const J2000: f64 = 2451545.0;
const ARCSEC: f64 = TAU / 1_296_000.0;
const CENTURY: f64 = 36525.0;

pub type Vec3 = [f64; 3];

pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn unit(a: Vec3) -> Vec3 {
    let n = dot(a, a).sqrt();
    [a[0] / n, a[1] / n, a[2] / n]
}

/// TT - UT in days, interpolated from the yearly table.
pub fn delta_t(jd_ut: f64) -> f64 {
    let year = 2000.0 + (jd_ut - 2451544.5) / 365.2425;
    let last = DELTA_T.len() - 2;
    let i = ((year.floor() as i32 - FIRST_YEAR).max(0) as usize).min(last);
    let f = year - (FIRST_YEAR + i as i32) as f64;
    (DELTA_T[i] as f64 + f * (DELTA_T[i + 1] - DELTA_T[i]) as f64) / 86400.0
}

const PQ_POLY: [[f64; 4]; 2] = [[5851.607687, -0.1189000, -0.00028913, 0.000000101], [-1600.886300, 1.1689818, -0.00000020, -0.000000437]];
const PQ_PERIODIC: [[f64; 5]; 8] = [
    [708.15, -5486.751211, -684.661560, 667.666730, -5523.863691],
    [2309.00, -17.127623, 2446.283880, -2354.886252, -549.747450],
    [1620.00, -617.517403, 399.671049, -428.152441, -310.998056],
    [492.20, 413.442940, -356.652376, 376.202861, 421.535876],
    [1183.00, 78.614193, -186.387003, 184.778874, -36.776172],
    [622.00, -180.732815, -316.800070, 335.321713, -145.278396],
    [882.00, -87.676083, 198.296701, -185.138669, -34.744450],
    [547.00, 46.140315, 101.135679, -120.972830, 22.885731],
];
const XY_POLY: [[f64; 4]; 2] = [[5453.282155, 0.4252841, -0.00037173, -0.000000152], [-73750.930350, -0.7675452, -0.00018725, 0.000000231]];
const XY_PERIODIC: [[f64; 5]; 14] = [
    [256.75, -819.940624, 75004.344875, 81491.287984, 1558.515853],
    [708.15, -8444.676815, 624.033993, 787.163481, 7774.939698],
    [274.20, 2600.009459, 1251.136893, 1251.296102, -2219.534038],
    [241.45, 2755.175630, -1102.212834, -1257.950837, -2523.969396],
    [2309.00, -167.659835, -2660.664980, -2966.799730, 247.850422],
    [492.20, 871.855056, 699.291817, 639.744522, -846.485643],
    [396.10, 44.769698, 153.167220, 131.600209, -1393.124055],
    [288.90, -512.313065, -950.865637, -445.040117, 368.526116],
    [231.10, -819.415595, 499.754645, 584.522874, 749.045012],
    [1610.00, -538.071099, -145.188210, -89.756563, 444.704518],
    [620.00, -189.793622, 558.116553, 524.429630, 235.934465],
    [157.87, -402.922932, -23.923029, -13.549067, 374.049623],
    [220.30, 179.516345, -165.405086, -210.157124, -171.330180],
    [1200.00, -9.814756, 9.344131, -44.919798, -22.899655],
];

fn series(t: f64, poly: &[[f64; 4]; 2], periodic: &[[f64; 5]]) -> (f64, f64) {
    let (mut a, mut b) = (0.0, 0.0);
    for row in periodic {
        let (s, c) = (TAU * t / row[0]).sin_cos();
        a += c * row[1] + s * row[3];
        b += c * row[2] + s * row[4];
    }
    let mut w = 1.0;
    for i in 0..4 {
        a += poly[0][i] * w;
        b += poly[1][i] * w;
        w *= t;
    }
    (a * ARCSEC, b * ARCSEC)
}

/// Pole of the ecliptic in the J2000 mean frame.
fn ecliptic_pole(t: f64) -> Vec3 {
    let (p, q) = series(t, &PQ_POLY, &PQ_PERIODIC);
    let w = (1.0 - p * p - q * q).max(0.0).sqrt();
    let (s, c) = (84381.406 * ARCSEC).sin_cos();
    [p, -q * c - w * s, -q * s + w * c]
}

/// Pole of the equator in the J2000 mean frame.
fn equator_pole(t: f64) -> Vec3 {
    let (x, y) = series(t, &XY_POLY, &XY_PERIODIC);
    [x, y, (1.0 - x * x - y * y).max(0.0).sqrt()]
}

/// Nutation in longitude and obliquity, radians. `t` is in Julian centuries of TT from J2000.
fn nutation(t: f64) -> (f64, f64) {
    let turn = |a: f64, b: f64| ((a + b * t) % 1_296_000.0) * ARCSEC;
    let args = [
        turn(485868.249036, 1717915923.2178),
        turn(1287104.79305, 129596581.0481),
        turn(335779.526232, 1739527262.8478),
        turn(1072260.70369, 1602961601.2090),
        turn(450160.398036, -6962890.5431),
    ];
    let (mut dp, mut de) = (0.0, 0.0);
    for (n, c) in NUTATION.iter().rev() {
        let arg: f64 = n.iter().zip(&args).map(|(&m, a)| m as f64 * a).sum();
        let (s, co) = arg.sin_cos();
        dp += (c[0] + c[1] * t) * s + c[2] * co;
        de += (c[3] + c[4] * t) * co + c[5] * s;
    }
    let unit = ARCSEC / 1e7;
    // Fixed offsets stand in for the planetary terms.
    (dp * unit - 0.135e-3 * ARCSEC, de * unit + 0.388e-3 * ARCSEC)
}

/// Everything about the Earth's orientation at one moment of terrestrial time.
pub struct Frame {
    /// Rows: axes of the mean ecliptic of date in the ICRS.
    ecliptic: [Vec3; 3],
    pub dpsi: f64,
    /// True obliquity, radians.
    pub eps: f64,
}

impl Frame {
    pub fn at(jd_tt: f64) -> Frame {
        let cy = (jd_tt - J2000) / CENTURY;
        let (pole_ecl, pole_eq) = (ecliptic_pole(cy), equator_pole(cy));
        let x = unit(cross(pole_eq, pole_ecl));
        let y = cross(pole_ecl, x);
        let (dpsi, deps) = nutation(cy);
        let eps = dot(pole_eq, pole_ecl).clamp(-1.0, 1.0).acos() + deps;
        // Frame bias of the ICRS against the mean J2000 frame.
        let (dx, de, dr) = (-0.016617 * ARCSEC, -0.0068192 * ARCSEC, -0.0146 * ARCSEC);
        let bias = |r: Vec3| [r[0] + r[1] * dr - r[2] * dx, -r[0] * dr + r[1] - r[2] * de, r[0] * dx + r[1] * de + r[2]];
        Frame { ecliptic: [bias(x), bias(y), bias(pole_ecl)], dpsi, eps }
    }

    /// Ecliptic longitude and latitude of date in radians, for a direction in the ICRS.
    pub fn to_ecliptic(&self, v: Vec3) -> (f64, f64) {
        let [a, b, c] = self.ecliptic;
        let (x, y, z) = (dot(a, v), dot(b, v), dot(c, v));
        (y.atan2(x) + self.dpsi, z.atan2((x * x + y * y).sqrt()))
    }
}

/// Apparent sidereal time at Greenwich in degrees, with the true obliquity in degrees.
pub fn sidereal_time(jd_ut: f64) -> (f64, f64) {
    let jd_tt = jd_ut + delta_t(jd_ut);
    let frame = Frame::at(jd_tt);
    let t = (jd_tt - J2000) / CENTURY;
    let d = jd_ut - J2000;
    let era = TAU * (0.7790572732640 + 0.00273781191135448 * d + d.rem_euclid(1.0));
    let poly = 0.014506 + t * (4612.156534 + t * (1.3915817 + t * (-0.00000044 + t * (-0.000029956 - t * 0.0000000368))));
    let node = ((450160.398036 - 6962890.5431 * t) % 1_296_000.0) * ARCSEC;
    let complementary = (2640.96e-6 * node.sin() + 63.52e-6 * (2.0 * node).sin()) * ARCSEC;
    let gast = era + poly * ARCSEC + frame.dpsi * frame.eps.cos() + complementary;
    (gast.to_degrees().rem_euclid(360.0), frame.eps.to_degrees())
}
