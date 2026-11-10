//! Vypíše všechny texty výkladu do jednoho souboru pro čtení a přepisování:
//! cargo run -p astro-core --example vyklady > dist/Vyklady-texty.txt
//! Věty jdou přes stejné funkce jako v aplikaci, takže výpis nemůže utéct od programu.
use astro_core::aspects::Aspect;
use astro_core::chart::{dignity, Body, Chart};
use astro_core::data::*;
use astro_core::interpret::*;

fn body(index: usize, sign: usize, offset: f64, house: Option<usize>, retro: bool) -> Body {
    let obj = &OBJECTS[index];
    Body {
        index,
        id: obj.id,
        name: obj.name,
        glyph: obj.glyph.to_string(),
        lon: sign as f64 * 30.0 + offset,
        lat: 0.0,
        speed: if retro { -1.0 } else { 1.0 },
        retro,
        sign,
        sign_name: SIGNS[sign],
        position: String::new(),
        house,
        dignity: dignity(obj, sign),
    }
}

fn chart(bodies: Vec<Body>) -> Chart {
    Chart { bodies, cusps: Some(std::array::from_fn(|i| i as f64 * 30.0)), jd_ut: 0.0 }
}

fn aspect(a: usize, b: usize, k: usize, orb: f64) -> Aspect {
    let d = &ASPECTS[k];
    Aspect {
        a,
        b,
        a_name: OBJECTS[a].name,
        b_name: OBJECTS[b].name,
        a_glyph: String::new(),
        b_glyph: String::new(),
        aspect: d.id,
        aspect_name: d.name,
        aspect_index: k,
        glyph: String::new(),
        color: d.color,
        orb,
        max_orb: d.orb,
        applying: false,
        power: 0.0,
    }
}

fn heading(s: &str) {
    println!("\n\n{0}\n{s}\n{0}", "=".repeat(78));
}

fn sub(s: &str) {
    println!("\n--- {s} ---\n");
}

fn main() {
    println!("ASTROLOG CZ: VŠECHNY TEXTY VÝKLADU");
    println!("Vygenerováno z programu. Oprava tady do programu sama nepřejde.");
    println!();
    println!("Jak aplikace výklad skládá:");
    println!("Vlastní odstavce má zatím jen Slunce v radixu. Ostatní věty se skládají z dílů v části 1.");
    println!("Část 2 ukazuje, co z nich vznikne. Pro přepis je podstatné:");
    println!("- Text domu u složených vět nezávisí na planetě: Měsíc i Saturn v 5. domě mají stejnou větu.");
    println!("- Text aspektu je u složených vět stejný pro všechny dvojice, mění se jen dosazené díly.");
    println!("- Tranzity, progrese, synastrie, složený horoskop a solární návrat nemají vlastní texty,");
    println!("  používají stejné díly (viz 1.6 a 2.3).");

    heading("ČÁST 1: DÍLY, ZE KTERÝCH SE VÝKLAD SKLÁDÁ");

    sub("1.1 Planety a body: co představují (14)");
    println!("Druhý tvar se dosazuje za aspektem (\"...splývají s ...\").\n");
    for (i, (nom, ins)) in PARTS.iter().enumerate() {
        println!("{}: {nom} / {ins}", OBJECTS[i].name);
    }

    sub("1.2 Znamení: jak se planeta projevuje (12)");
    for (i, s) in SIGN_STYLE.iter().enumerate() {
        println!("{}: {s}", SIGNS[i]);
    }
    println!("\nPrvních 10° znamení přidává slovo \"velmi\" (\"se projevují velmi rázně...\").");

    sub("1.3 Znamení: po čem planeta touží (12)");
    for (i, s) in SIGN_DESIRE.iter().enumerate() {
        println!("{}: {s}", SIGNS[i]);
    }

    sub("1.4 Domy: oblast života (12)");
    println!("Věta zní: \"Nejčastěji se to projeví v oblasti ...\" a je stejná pro každou planetu.");
    println!("Ascendent a Medium coeli větu o domě nemají.\n");
    for (i, s) in HOUSE_AREA.iter().enumerate() {
        println!("{}. dům: {s}", i + 1);
    }
    let r = placements(&chart(vec![body(2, 0, 15.0, Some(3), true)]));
    println!("\nZpětný pohyb (mimo Severní uzel) větu mění, příklad:\n{}", r.items[0].text);

    sub("1.5 Aspekty");
    println!("Slovo se volí podle přesnosti: orb do 2,5° / do 5° / nad 5°.\n");
    for (i, (v, words)) in ASPECT_VERB.iter().enumerate() {
        println!("{}:", ASPECTS[i].name);
        for w in words {
            println!("  {}", v.replace("{}", w));
        }
        if !ASPECT_NOTE[i].is_empty() {
            println!("  Dodatek: {}", ASPECT_NOTE[i]);
        }
    }
    println!("\nMenší aspekty (Kvinkunx, Polosextil, Polokvadrát, Seskvikvadrát) mají jednu společnou větu:");
    println!("  {ASPECT_MINOR}");

    sub("1.6 Doplňkové věty");
    println!("Vládce znamení: Je to výrazná část osobnosti.");
    println!("Vyhnanství: V osobnosti to hraje spíše menší roli.");
    println!("Povýšení: Tuto stránku lze projevit snadno.");
    println!("Pád: Tuto stránku je těžší projevit.");
    println!("Tranzit nebo progrese, konjunkce: Tato část osobnosti bude silně ovlivněna.");
    println!("Synastrie, planeta v povýšení: Působí to harmonicky.");
    println!("Synastrie, planeta v pádu: Působí to spíše nesouladně.");

    heading("ČÁST 2: VĚTY, KTERÉ Z DÍLŮ VZNIKNOU");

    sub("2.1 Planeta ve znamení (14 × 12 = 168)");
    for i in 0..OBJECTS.len() {
        println!("\n{}", OBJECTS[i].name.to_uppercase());
        for s in 0..12 {
            let sec = placements(&chart(vec![body(i, s, 15.0, None, false)]));
            println!("{} {}: {}", OBJECTS[i].name, SIGNS_LOC[s], sec.items[0].text);
        }
    }

    sub("2.2 Aspekty mezi dvojicemi v radixu (91 dvojic × 5 aspektů, střední síla)");
    println!("U přesnějšího nebo volnějšího aspektu se mění jen slovo podle 1.5.");
    for a in 0..OBJECTS.len() {
        for b in a + 1..OBJECTS.len() {
            println!("\n{} – {}", OBJECTS[a].name, OBJECTS[b].name);
            let list: Vec<Aspect> = (0..MAJOR_ASPECTS).map(|k| aspect(a, b, k, 3.0)).collect();
            for (k, item) in natal_aspects(&list).items.iter().enumerate() {
                println!("{}: {}", ASPECTS[k].name, item.text);
            }
        }
    }

    sub("2.3 Příklady z ostatních pohledů");
    println!("Tranzity a progrese (všechny kombinace se tvoří stejně, jen s jinými díly):");
    let moving: Vec<Aspect> = [(6, 0, 2), (5, 1, 3), (9, 9, 1), (7, 12, 0)].iter().map(|&(a, b, k)| aspect(a, b, k, 1.0)).collect();
    for item in active_moving(&moving, "", TRANSIT).items {
        println!("{}\n  {}", item.title, item.text);
    }
    let (jana, petr) = (astro_core::names::name_of("Jana", ""), astro_core::names::name_of("Petr", ""));
    println!("\nSynastrie, aspekty mezi lidmi:");
    let rel = [aspect(3, 4, 0, 1.0), aspect(1, 6, 2, 3.0)];
    for item in relation_aspects(&rel, &jana, &petr).items {
        println!("{}\n  {}", item.title, item.text);
    }
    println!("\nSynastrie, planeta jednoho v domě druhého:");
    let host = chart(vec![body(0, 0, 15.0, Some(1), false)]);
    let guest = chart(vec![body(3, 6, 15.0, Some(7), false), body(6, 3, 15.0, Some(4), false)]);
    for item in overlay(&guest, &host, &petr, &jana, true).items {
        println!("{}\n  {}", item.title, item.text);
    }
    println!("\nSložený horoskop a solární návrat používají věty z 2.1 a 2.2.");
}
