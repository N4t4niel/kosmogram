//! House cusps. Every system is built from the sphere: a house circle is a plane through the observer,
//! and its cusp is where that plane cuts the ecliptic.
use crate::earth::{cross, dot, sidereal_time, unit, Vec3};

const FAILED: &str = "Pro tak severní nebo jižní místo tento systém domů nejde spočítat. Zvolte v Nastavení například Rovné domy nebo Celá znamení.";

fn norm(x: f64) -> f64 {
    x.rem_euclid(360.0)
}

/// Signed shortest difference a - b in degrees.
fn diff(a: f64, b: f64) -> f64 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

struct Sky {
    armc: f64,
    lat: f64,
    eps: f64,
}

impl Sky {
    /// Longitude of the ecliptic point with right ascension `ra`.
    fn lon_from_ra(&self, ra: f64) -> f64 {
        let r = ra.to_radians();
        norm(r.sin().atan2(r.cos() * self.eps.to_radians().cos()).to_degrees())
    }

    fn ra_from_lon(&self, lon: f64) -> f64 {
        let l = lon.to_radians();
        norm((l.sin() * self.eps.to_radians().cos()).atan2(l.cos()).to_degrees())
    }

    fn dec_from_lon(&self, lon: f64) -> f64 {
        (self.eps.to_radians().sin() * lon.to_radians().sin()).asin().to_degrees()
    }

    /// Ecliptic point rising at sidereal angle `ramc` for a horizon of pole height `pole`.
    fn rising(&self, ramc: f64, pole: f64) -> f64 {
        let (t, e, p) = (ramc.to_radians(), self.eps.to_radians(), pole.to_radians());
        norm(t.cos().atan2(-(t.sin() * e.cos() + p.tan() * e.sin())).to_degrees())
    }

    fn mc(&self) -> f64 {
        self.lon_from_ra(self.armc)
    }

    fn asc(&self) -> f64 {
        self.rising(self.armc, self.lat)
    }

    /// Ascensional difference of a declination; `None` where the point never rises or sets.
    fn asc_diff(&self, dec: f64) -> Option<f64> {
        let x = self.lat.to_radians().tan() * dec.to_radians().tan();
        (x.abs() <= 1.0).then(|| x.asin().to_degrees())
    }

    /// Placidus: a cusp sits at a fixed fraction of the semi-arc of its own degree, so iterate.
    fn placidus_cusp(&self, fraction: f64, above: bool, start: f64) -> Option<f64> {
        let mut lon = start;
        for _ in 0..100 {
            let ad = self.asc_diff(self.dec_from_lon(lon))?;
            let ra = if above { self.armc + fraction * (90.0 + ad) } else { self.armc + 180.0 - fraction * (90.0 - ad) };
            let next = self.lon_from_ra(ra);
            let change = diff(next, lon).abs();
            lon = next;
            if change < 1e-9 {
                return Some(lon);
            }
        }
        None
    }

    /// Axes of the local sky in equatorial coordinates of date: south, east, zenith.
    fn local_axes(&self) -> (Vec3, Vec3, Vec3) {
        let (t, p) = (self.armc.to_radians(), self.lat.to_radians());
        let zenith = [p.cos() * t.cos(), p.cos() * t.sin(), p.sin()];
        let east = [-t.sin(), t.cos(), 0.0];
        (cross(east, zenith), east, zenith)
    }

    /// Where the circle through the north and south points and through `point` cuts the ecliptic,
    /// on the eastern side.
    fn circle_cusp(&self, point: Vec3) -> f64 {
        let (south, east, _) = self.local_axes();
        let normal = cross(south, point);
        let e = self.eps.to_radians();
        let ecliptic_pole = [0.0, -e.sin(), e.cos()];
        let mut cut = unit(cross(normal, ecliptic_pole));
        if dot(cut, east) < 0.0 {
            cut = [-cut[0], -cut[1], -cut[2]];
        }
        let x = cut[0];
        let y = cut[1] * e.cos() + cut[2] * e.sin();
        norm(y.atan2(x).to_degrees())
    }
}

/// Cusps of houses 1 to 12 with the ascendant and midheaven.
pub fn houses(jd_ut: f64, lat: f64, lon: f64, system: u8) -> Result<([f64; 12], f64, f64), String> {
    let (gast, eps) = sidereal_time(jd_ut);
    Ok(houses_at(norm(gast + lon), lat, eps, system)?)
}

/// Ascendant and midheaven alone, which searches need thousands of times.
pub fn angles(jd_ut: f64, lat: f64, lon: f64, system: u8) -> (f64, f64) {
    let (gast, eps) = sidereal_time(jd_ut);
    let sky = Sky { armc: norm(gast + lon), lat, eps };
    polar_angles(sky.asc(), sky.mc(), lat, eps, system)
}

/// Sidereal angle of the birth midheaven moved `arc` degrees along the ecliptic.
fn armc_by_arc(natal_jd: f64, lon: f64, arc: f64, eps: f64) -> f64 {
    let (gast, natal_eps) = sidereal_time(natal_jd);
    let mc = Sky { armc: norm(gast + lon), lat: 0.0, eps: natal_eps }.mc();
    Sky { armc: 0.0, lat: 0.0, eps }.ra_from_lon(norm(mc + arc))
}

/// Cusps recomputed from the birth midheaven moved by `arc`, with the obliquity of `jd_ut`.
pub fn houses_by_arc(natal_jd: f64, jd_ut: f64, arc: f64, lat: f64, lon: f64, system: u8) -> Result<([f64; 12], f64, f64), String> {
    let (_, eps) = sidereal_time(jd_ut);
    houses_at(armc_by_arc(natal_jd, lon, arc, eps), lat, eps, system)
}

/// Ascendant and midheaven alone for `houses_by_arc`.
pub fn angles_by_arc(natal_jd: f64, jd_ut: f64, arc: f64, lat: f64, lon: f64, system: u8) -> (f64, f64) {
    let (_, eps) = sidereal_time(jd_ut);
    let sky = Sky { armc: armc_by_arc(natal_jd, lon, arc, eps), lat, eps };
    polar_angles(sky.asc(), sky.mc(), lat, eps, system)
}

/// Inside the polar circles the ascendant formula can return the setting point. Regiomontanus and
/// Campanus then count the houses the other way round the zodiac, from the opposite midheaven.
fn polar_angles(asc: f64, mc: f64, lat: f64, eps: f64, system: u8) -> (f64, f64) {
    if lat.abs() < 90.0 - eps || diff(asc, mc) >= 0.0 {
        return (asc, mc);
    }
    (norm(asc + 180.0), if matches!(system, b'R' | b'C') { norm(mc + 180.0) } else { mc })
}

/// The same from sidereal angle and true obliquity, in degrees.
pub fn houses_at(armc: f64, lat: f64, eps: f64, system: u8) -> Result<([f64; 12], f64, f64), String> {
    let sky = Sky { armc, lat, eps };
    let (asc, mc) = polar_angles(sky.asc(), sky.mc(), lat, eps, system);
    let polar = lat.abs() >= 90.0 - eps;
    let mut c = [0.0; 12];
    c[0] = asc;
    c[9] = mc;
    // Cusps 11, 12, 2 and 3; the opposite ones follow.
    let mut set = |c11: f64, c12: f64, c2: f64, c3: f64| {
        c[10] = c11;
        c[11] = c12;
        c[1] = c2;
        c[2] = c3;
    };
    match system {
        b'P' => {
            if polar {
                return Err(FAILED.into());
            }
            let a = diff(asc, mc);
            let step = |k: f64| norm(mc + a * k / 3.0);
            let found = (|| {
                Some((
                    sky.placidus_cusp(1.0 / 3.0, true, step(1.0))?,
                    sky.placidus_cusp(2.0 / 3.0, true, step(2.0))?,
                    sky.placidus_cusp(2.0 / 3.0, false, norm(asc + 30.0))?,
                    sky.placidus_cusp(1.0 / 3.0, false, norm(asc + 60.0))?,
                ))
            })();
            let (c11, c12, c2, c3) = found.ok_or(FAILED)?;
            set(c11, c12, c2, c3);
        }
        b'K' => {
            let ad = sky.asc_diff(sky.dec_from_lon(mc)).filter(|_| !polar).ok_or(FAILED)?;
            let a = (90.0 + ad) / 3.0;
            let at = |k: f64| sky.rising(armc + k * a, lat);
            set(at(-2.0), at(-1.0), at(1.0), at(2.0));
        }
        b'R' => {
            let east = |h: f64| {
                let t = (armc + h).to_radians();
                [t.cos(), t.sin(), 0.0]
            };
            set(sky.circle_cusp(east(30.0)), sky.circle_cusp(east(60.0)), sky.circle_cusp(east(120.0)), sky.circle_cusp(east(150.0)));
        }
        b'C' => {
            let (_, east, zenith) = sky.local_axes();
            let at = |alt: f64| {
                let (s, co) = alt.to_radians().sin_cos();
                [co * east[0] + s * zenith[0], co * east[1] + s * zenith[1], co * east[2] + s * zenith[2]]
            };
            set(sky.circle_cusp(at(60.0)), sky.circle_cusp(at(30.0)), sky.circle_cusp(at(-30.0)), sky.circle_cusp(at(-60.0)));
        }
        b'O' => {
            let upper = norm(asc - mc) / 3.0;
            let lower = norm(mc + 180.0 - asc) / 3.0;
            set(norm(mc + upper), norm(mc + 2.0 * upper), norm(asc + lower), norm(asc + 2.0 * lower));
        }
        b'B' => {
            let asc_ra = sky.ra_from_lon(asc);
            let upper = norm(asc_ra - armc) / 3.0;
            let lower = norm(armc + 180.0 - asc_ra) / 3.0;
            let lon = |ra: f64| sky.lon_from_ra(ra);
            set(lon(armc + upper), lon(armc + 2.0 * upper), lon(asc_ra + lower), lon(asc_ra + 2.0 * lower));
        }
        b'E' | b'W' => {
            let first = if system == b'E' { asc } else { (asc / 30.0).floor() * 30.0 };
            for i in 0..12 {
                c[i] = norm(first + i as f64 * 30.0);
            }
            return Ok((c, asc, mc));
        }
        _ => return Err("Neznámý systém domů".into()),
    }
    // Beyond the polar circles a house circle can cut the ecliptic on the far side; the cusps still
    // have to run in order from the midheaven through the ascendant, in whichever direction that is.
    if polar {
        let inside = |x: f64, from: f64, to: f64| (0.0..=1.0).contains(&(diff(x, from) / diff(to, from)));
        for (i, from, to) in [(10, mc, asc), (11, mc, asc), (1, asc, mc + 180.0), (2, asc, mc + 180.0)] {
            if !inside(c[i], from, to) {
                c[i] = norm(c[i] + 180.0);
            }
        }
    }
    for (a, b) in [(3, 9), (4, 10), (5, 11), (6, 0), (7, 1), (8, 2)] {
        c[a] = norm(c[b] + 180.0);
    }
    Ok((c, asc, mc))
}
