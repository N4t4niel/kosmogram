# Cizí části programu a jejich licence

Kosmogram je česká verze vycházející z programu Astrolog 8.00 Waltera D. Pullena, není to původní program. Jako celek je pod licencí GNU GPL verze 2 nebo novější, protože z Astrologu vychází. Text licence je v `LICENSE`.

Sestavený program obsahuje knihovny pod licencí Apache 2.0 (například `tao`, `ring`, `sync_wrapper`), které se slučují jen s GPL verze 3. Sestavený program se proto šíří podle GPL verze 3, což volba „verze 2 nebo novější“ dovoluje. Text GPL 3 je v `licenses/GPL-3.0.txt`.

| Část | Kde je | Autor | Licence |
|---|---|---|---|
| Astrolog 8.00: výchozí kód, tabulky a složené věty výkladu | převedeno do `core/src`, hlavně `data.rs`, `interpret.rs`, `chart.rs`, `aspects.rs`, `search.rs` | Walter D. Pullen, astrolog.org | GPL 2 nebo novější. Upozornění autora je beze změny v `licenses/Astrolog-NOTICE.txt` a musí zůstat zachované. |
| Efemeridy | `core/data/ephem.bin`, `core/src/deltat_table.rs` | NASA JPL: DE440, Horizons (Chiron, ΔT) | volně zveřejněná data JPL, přepočtená do vlastního formátu skriptem `tools/build_ephem.py` |
| Řada nutace a koeficienty precese, IAU 2000B a Vondrák a kol. 2011 | `core/src/nut_table.rs`, `core/src/earth.rs` | IAU SOFA a projekt ERFA | BSD, `licenses/ERFA-BSD.txt` |
| Data míst | `core/data/places.tsv.gz` | GeoNames, geonames.org. Upraveno: výběr obydlených míst, česká jména z alternativních názvů a jen potřebné údaje, viz `tools/build_places.py`. | CC BY 4.0, https://creativecommons.org/licenses/by/4.0/ |
| Časová pásma | přes knihovnu `chrono-tz` | IANA Time Zone Database | public domain |
| Letní čas v Československu | `core/src/timeconv.rs` | podle Sbírky zákonů (181/1946, 54/1949, 36/1979, 31/1980), pro kontrolu přehled kalendar.beda.cz | údaje z právních předpisů |
| Písmo Astronomicon | `ui/fonts` | Roberto Corona | SIL OFL 1.1, `ui/fonts/Astronomicon-OFL-License.txt` |
| Knihovny Rust a Tauri | `Cargo.lock` | autoři jednotlivých balíčků | hlavně MIT a Apache 2.0, dále BSD, Zlib, Unicode a MPL 2.0. Seznam a úplné texty jsou v `licenses/THIRD-PARTY.txt`, vytváří ho `tools/build_licenses.py`. |

Vlastní části programu: český přepis výpočtů, rozhraní, nápověda, ikona (`desktop/icons/icon.svg`) a výklady v `core/data/vyklady`. Výklady jsou nově napsané české texty. Obecně známé astrologické významy jsou v nich popsané vlastními slovy, nic není přeložené ani převyprávěné z cizího textu.

Výpočty domů, zdánlivých poloh a středního uzlu jsou napsané podle publikovaných vzorců.

Program neobsahuje žádný kód ani data Swiss Ephemeris a PLACALC, které původní Astrolog používá. Upozornění na ně v `licenses/Astrolog-NOTICE.txt` zůstává, protože je součástí upozornění autora Astrologu.

Soubory `core/tests/fixtures` slouží jen k testům a do programu se nesestavují. Jsou to číselné výsledky původního Astrologu, NASA JPL Horizons a Swiss Ephemeris (`swiss_*.txt`), se kterými testy porovnávají výpočty.

Instalátor kopíruje složku `licenses` a soubory `LICENSE` a `NOTICE.md` vedle programu. V programu jsou autoři a licence v nápovědě, téma Autoři a licence.
