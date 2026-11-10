"""Kontrola textů výkladů: python tools/kontrola_vykladu.py core/data/vyklady/mesic.txt [...]"""
import re
import sys

BODIES = "Slunce Měsíc Merkur Venuše Mars Jupiter Saturn Uran Neptun Pluto Chiron".split() + ["Severní uzel", "Ascendent", "Medium coeli"]
SIGNS = "Beran Býk Blíženci Rak Lev Panna Váhy Štír Střelec Kozoroh Vodnář Ryby".split()
ASPECTS = "Konjunkce Opozice Kvadratura Trigon Sextil Kvinkunx Polosextil Polokvadrát Seskvikvadrát".split()
HOUSE = re.compile(r"^(1[0-2]|[1-9])\. dům$")
ONLY = {
    ("Slunce", "Merkur"): {"Konjunkce", "Polosextil"},
    ("Slunce", "Venuše"): {"Konjunkce", "Polosextil", "Polokvadrát"},
    ("Merkur", "Venuše"): {"Konjunkce", "Polosextil", "Polokvadrát", "Sextil"},
    ("Ascendent", "Medium coeli"): {"Kvadratura", "Trigon", "Sextil"},
}
NO_HOUSE = {"Ascendent", "Medium coeli"}
NO_RETRO = {"Slunce", "Měsíc", "Severní uzel", "Ascendent", "Medium coeli"}
BANNED = ["—", "–", "  ", " ,", "tradičně", "tradiční astrolog", "starší knihy", "starší astrolog", "zemře", "rakovin", " vy ", " vás ", " vám ", " váš ", "Vaše", "Váš"]
LIMITS = {"radix": 700, "other": 520}


PHASES = ["Novoluní", "Dorůstající srpek", "První čtvrť", "Dorůstající Měsíc", "Úplněk", "Couvající Měsíc", "Poslední čtvrť", "Couvající srpek"]
OVERVIEW = {
    "Fáze Měsíce": lambda p: len(p) == 2 and p[1] in PHASES,
    "Živel": lambda p: len(p) == 3 and p[1] in ("Oheň", "Země", "Vzduch", "Voda") and p[2] in ("převaha", "nedostatek"),
    "Kvalita": lambda p: len(p) == 3 and p[1] in ("Kardinální", "Pevná", "Proměnlivá") and p[2] == "převaha",
    "Polokoule": lambda p: len(p) == 3 and p[1] in ("Východ", "Západ", "Sever", "Jih") and p[2] == "převaha",
    "Obrazec": lambda p: len(p) == 2 and p[1] in ("Velký trigon", "Velký kříž", "T-kvadrát", "Jod", "Drak", "Seskupení planet"),
    "Hrot": lambda p: len(p) == 3 and HOUSE.match(p[1]) and p[2] in SIGNS,
    "Vládce": lambda p: len(p) == 3 and HOUSE.match(p[1]) and HOUSE.match(p[2]),
}


def key_error(p):
    if p[0] in OVERVIEW:
        return None if OVERVIEW[p[0]](p) else "špatný klíč"
    if p[0] in ("Tranzit", "Progrese"):
        if len(p) != 4 or p[1] not in BODIES or p[2] not in BODIES or p[3] not in ASPECTS:
            return "špatný klíč"
        return None
    if p[0] == "Progrese denní":
        if len(p) != 4 or p[1] not in ("Ascendent", "Medium coeli") or p[2] not in BODIES or p[3] not in ASPECTS:
            return "špatný klíč"
        return None
    if p[0] == "Synastrie":
        if len(p) != 4 or p[1] not in BODIES or p[2] not in BODIES or p[3] not in ASPECTS:
            return "špatný klíč"
        return None if BODIES.index(p[1]) <= BODIES.index(p[2]) else "špatné pořadí dvojice"
    if p[0] in ("Synastrie dům", "Složený", "Solární", "Lunární"):
        if len(p) == 3 and p[1] in BODIES and (HOUSE.match(p[2]) or p[2] in SIGNS):
            return None
        if p[0] != "Synastrie dům" and len(p) == 4 and p[1] in BODIES and p[2] in BODIES and p[3] in ASPECTS:
            if BODIES.index(p[1]) >= BODIES.index(p[2]):
                return "špatné pořadí dvojice"
            return "aspekt nemůže nastat" if (p[1], p[2]) in ONLY and p[3] not in ONLY[(p[1], p[2])] else None
        return "špatný klíč"
    if p[0] not in BODIES:
        return "neznámé těleso"
    if len(p) == 2:
        if p[1] in SIGNS:
            return None
        if HOUSE.match(p[1]):
            return "dům u tohoto bodu neexistuje" if p[0] in NO_HOUSE else None
        if p[1] == "Zpětný pohyb":
            return "zpětný pohyb tu neexistuje" if p[0] in NO_RETRO else None
        return "špatný klíč"
    if len(p) == 3:
        if p[1] not in BODIES or p[2] not in ASPECTS:
            return "špatný klíč"
        if BODIES.index(p[0]) >= BODIES.index(p[1]):
            return "špatné pořadí dvojice"
        if (p[0], p[1]) in ONLY and p[2] not in ONLY[(p[0], p[1])]:
            return "aspekt nemůže nastat"
        return None
    return "špatný počet částí"


# Samples of the forms produced by core/src/names.rs.
MALE = dict(zip(("", "g", "d", "a", "l", "i"), "Petr Petra Petrovi Petra Petrovi Petrem".split()), female=False)
MALE2 = dict(zip(("", "g", "d", "a", "l", "i"), "Tomáš Tomáše Tomášovi Tomáše Tomášovi Tomášem".split()), female=False)
FEMALE = dict(zip(("", "g", "d", "a", "l", "i"), "Jana Jany Janě Janu Janě Janou".split()), female=True)
FEMALE2 = dict(zip(("", "g", "d", "a", "l", "i"), "Věra Věry Věře Věru Věře Věrou".split()), female=True)
TOKEN = re.compile(r"\{([^{}]*)\}")


def fill(text, a, b):
    def one(m):
        t = m.group(1)
        if ":" in t:
            who, forms = t.split(":", 1)
            male, female = forms.split("|")
            return female if {"1": a["female"], "2": b["female"], "p": a["female"] and b["female"]}[who] else male
        return (a if t[0] == "1" else b)[t[1:]]
    text = TOKEN.sub(one, text)
    text = re.sub(r"\b([vV]) (?=[VF])", r"\1e ", text)
    return re.sub(r"\b([sSzZ]) (?=[SŠZŽ])", r"\1e ", text)


def template_error(text, a, b):
    if text.count("{") != text.count("}"):
        return "nespárované závorky šablony"
    for t in TOKEN.findall(text):
        if ":" in t:
            who, forms = t.split(":", 1)
            if who not in ("1", "2", "p") or forms.count("|") != 1:
                return f"špatná šablona: {t}"
        elif t[:1] not in ("1", "2") or t[1:] not in ("", "g", "d", "a", "l", "i"):
            return f"špatná šablona: {t}"
    return None


# "člověk" as the subject of the reading; general phrases like "každý člověk" may stay.
LEFT_PERSON = re.compile(r"(?<!každý )(?<!Každý )(?<!žádný )\b[Čč]lověk\w*")
SHOW_FEMALE = False


def check(path):
    bad, seen, count = [], {}, 0
    for n, line in enumerate(open(path, encoding="utf-8-sig"), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(" | ")
        if len(parts) < 2:
            bad.append((n, "chybí oddělovač"))
            continue
        *key, text = parts
        count += 1
        err = key_error(key)
        if err:
            bad.append((n, f"{err}: {' | '.join(key)}"))
        joined = " | ".join(key)
        if joined in seen:
            bad.append((n, f"duplicita s řádkem {seen[joined]}"))
        seen[joined] = n
        if key[0] in ("Synastrie", "Synastrie dům", "Složený"):
            for pair in ((MALE, FEMALE), (FEMALE, MALE), (FEMALE, FEMALE2), (MALE, MALE2)):
                problem = template_error(text, *pair)
                if problem:
                    bad.append((n, problem))
                    break
            text = fill(text, MALE, FEMALE)
        else:
            # Personal texts name one person: {1} and its cases, {1:muž|žena}. Checked as a man and as a woman.
            if re.search(r"\{(2|p)", text):
                bad.append((n, "osobní text zná jen jednoho člověka, {1}"))
            for who in (MALE, FEMALE):
                problem = template_error(text, who, who)
                if problem:
                    bad.append((n, problem))
                    break
            if SHOW_FEMALE:
                print(f"{n}: {fill(text, FEMALE, FEMALE)}")
            if LEFT_PERSON.search(text):
                bad.append((n, "zůstalo slovo člověk, má tu být jméno {1} nebo zájmeno"))
            text = fill(text, MALE, MALE)
        if not text.endswith(".") or not text[0].isupper():
            bad.append((n, "text nezačíná velkým písmenem nebo nekončí tečkou"))
        padded = f" {text} "
        for word in BANNED:
            if word in padded or word.lower() in padded.lower() and word.strip()[0].islower() and len(word) > 4:
                bad.append((n, f"zakázané: {word.strip() or repr(word)}"))
        limit = LIMITS["radix"] if len(key) <= 3 and key[0] in BODIES else LIMITS["other"]
        if len(text) > limit:
            bad.append((n, f"příliš dlouhé ({len(text)})"))
        if len(text) < 40:
            bad.append((n, f"příliš krátké ({len(text)})"))
    for n, msg in bad:
        print(f"{path}:{n}: {msg}")
    print(f"{path}: {count} textů, {len(bad)} chyb")
    return bad


if __name__ == "__main__":
    # --zena vypíše osobní texty, jak je program napíše pro Janu, ke kontrole rodu a pádů.
    SHOW_FEMALE = "--zena" in sys.argv
    sys.exit(1 if sum(len(check(p)) for p in sys.argv[1:] if not p.startswith("--")) else 0)
