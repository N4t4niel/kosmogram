"""Builds core/data/places.tsv.gz from GeoNames dumps (CC BY 4.0, https://www.geonames.org/).
Usage: python -I build_places.py <dir with dumps> <output file>"""
import csv, gzip, io, sys, zipfile
src, out = sys.argv[1], sys.argv[2]
csv.field_size_limit(10**7)
def rows(zname, member):
    with zipfile.ZipFile(f"{src}/{zname}") as z, z.open(member) as f:
        for line in io.TextIOWrapper(f, "utf-8"):
            yield line.rstrip("\n").split("\t")

places = {}
SKIP = {"PPLH", "PPLQ", "PPLW", "PPLCH"}
def add(r):
    if r[6] == "P" and r[7] not in SKIP and r[17]:
        places[r[0]] = r
for r in rows("cities1000.zip", "cities1000.txt"): add(r)
for z in ("CZ", "SK"):
    for r in rows(f"{z}.zip", f"{z}.txt"): add(r)
# Neighbours in full, except city districts (PPLX), which would bury the villages in noise.
NEIGHBOURS = ("DE", "AT", "PL", "HU", "UA")
for z in NEIGHBOURS:
    for r in rows(f"{z}.zip", f"{z}.txt"):
        if r[7] != "PPLX": add(r)

countries, country_ids = {}, {}
for line in open(f"{src}/countryInfo.txt", encoding="utf-8"):
    if line.startswith("#"): continue
    c = line.rstrip("\n").split("\t")
    countries[c[0]] = c[4]; country_ids[c[16]] = c[0]
admin1, admin2 = {}, {}
for line in open(f"{src}/admin1CodesASCII.txt", encoding="utf-8"):
    c = line.rstrip("\n").split("\t"); admin1[c[0]] = (c[1], c[3])
for line in open(f"{src}/admin2Codes.txt", encoding="utf-8"):
    c = line.rstrip("\n").split("\t"); admin2[c[0]] = (c[1], c[3])
admin_ids = {v[1]: None for v in admin1.values()} | {v[1]: None for v in admin2.values()}

cs, best = {}, {}
for r in rows("alt.zip", "alternateNamesV2.txt"):
    gid, lang, name = r[1], r[2], r[3]
    pref, short, coll, hist = (r + [""] * 8)[4:8]
    if lang != "cs" or coll == "1" or hist == "1" or short == "1": continue
    if gid in places or gid in country_ids or gid in admin_ids:
        score = 2 if pref == "1" else 1
        if score > best.get(gid, 0):
            best[gid] = score; cs[gid] = name
for iso_gid, iso in country_ids.items():
    if iso_gid in cs: countries[iso] = cs[iso_gid]
countries["CZ"] = "Česko"

n = 0
with gzip.open(out, "wt", encoding="utf-8", compresslevel=9) as f:
    for gid, r in places.items():
        cc = r[8]
        name = cs.get(gid, r[1])
        aliases = sorted({a for a in (r[1], r[2]) if a and a != name})
        a2 = admin2.get(f"{cc}.{r[10]}.{r[11]}")
        a1 = admin1.get(f"{cc}.{r[10]}")
        region = ""
        if cc in ("CZ", "SK") and a2:
            region = cs.get(a2[1], a2[0])
        elif cc in NEIGHBOURS and a1:
            # Villages repeat a lot there, so name both the district and the land.
            parts = [cs.get(x[1], x[0]) for x in (a2, a1) if x]
            region = ", ".join(dict.fromkeys(parts))
        elif a1:
            region = cs.get(a1[1], a1[0])
        pop = int(r[14] or 0)
        f.write("\t".join([name, "|".join(aliases), countries.get(cc, cc), region, f"{float(r[4]):.4f}", f"{float(r[5]):.4f}", r[17], str(pop)]) + "\n")
        n += 1
print(n, "places")
