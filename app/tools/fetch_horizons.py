"""Downloads apparent places from JPL Horizons for the independent check in core/tests/horizons.rs.

Usage: python tools/fetch_horizons.py
"""
import re
import urllib.parse
import urllib.request
from pathlib import Path

BODIES = {0: "10", 1: "301", 2: "1", 3: "2", 4: "4", 5: "5", 6: "6", 7: "7", 8: "8", 9: "9", 15: "2060;"}
SPANS = [("1800-01-05", "1800-04-05", "5 d"), ("1800-05-01", "2399-11-01", "211 d")]


def fetch(command, start, stop, step):
    query = {
        "format": "text", "COMMAND": f"'{command}'", "OBJ_DATA": "'NO'", "MAKE_EPHEM": "'YES'",
        "EPHEM_TYPE": "'OBSERVER'", "CENTER": "'500@399'", "START_TIME": f"'{start}'", "STOP_TIME": f"'{stop}'",
        "STEP_SIZE": f"'{step}'", "QUANTITIES": "'31'", "TIME_TYPE": "'TT'", "CAL_FORMAT": "'JD'",
        "ANG_FORMAT": "'DEG'", "EXTRA_PREC": "'YES'", "CSV_FORMAT": "'YES'",
    }
    url = "https://ssd.jpl.nasa.gov/api/horizons.api?" + urllib.parse.urlencode(query)
    text = urllib.request.urlopen(url, timeout=300).read().decode()
    if "$$SOE" not in text:
        raise SystemExit(text[:800])
    return re.findall(r"^\s*(\d+\.\d+),[^,]*,[^,]*,\s*([\d.]+),\s*(-?[\d.]+),", text.split("$$SOE")[1].split("$$EOE")[0], re.M)


lines = ["# JPL Horizons, geocentric apparent ecliptic of date, TT: body, Julian day, longitude, latitude"]
for body, command in BODIES.items():
    for span in SPANS:
        rows = fetch(command, *span)
        print(body, span[0], len(rows))
        lines += [f"{body} {jd} {lon} {lat}" for jd, lon, lat in rows]
path = Path(__file__).resolve().parent.parent / "core/tests/fixtures/horizons.txt"
path.parent.mkdir(exist_ok=True)
path.write_text("\n".join(lines) + "\n", encoding="utf8")
