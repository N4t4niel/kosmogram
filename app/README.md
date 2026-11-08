# Kosmogram

Astrologický program v češtině pro Windows: horoskop, obloha, tranzity, progrese, solární návrat, kalendář, synastrie a složený horoskop, s českými výklady a nápovědou. Vychází z programu Astrolog 8.00 Waltera D. Pullena, výpočty jsou přepsané do Rustu a polohy těles se počítají z dat NASA JPL přímo v programu, bez internetu.

## Instalace

Instalátor `Kosmogram_<verze>_x64-setup.exe` je ve vydáních (Releases) tohoto repozitáře. Instaluje jen pro přihlášeného uživatele, bez hesla správce, a když WebView2 chybí, doinstaluje ho. Lidé a nastavení jsou v `%APPDATA%\kosmogram`, aktualizace ani odinstalace je nemažou. Program si při spuštění sám zjistí novou verzi a nabídne ji.

## Struktura

- `core/`: výpočty, české texty a kreslení kola. Polohy těles počítá `ephem.rs` z `core/data/ephem.bin`.
- `core/data/vyklady/`: české výklady po klíčích, vestavěné do programu. Pravidla psaní jsou v `PRAVIDLA.md` a `STYL.md`.
- `desktop/`: aplikace Tauri, ukládání lidí do `lide.json`, denní kopie ve složce `zalohy`, aktualizace přes GitHub.
- `ui/`: rozhraní a nápověda (HTML, CSS, JS bez sestavování).
- `core/tests/`: testy, mimo jiné porovnání s uloženými výsledky původního Astrologu, NASA JPL Horizons a Swiss Ephemeris v `core/tests/fixtures`.
- `tools/`: sestavení efemerid (`build_ephem.py`), seznamu míst z GeoNames (`build_places.py`), přehledu licencí (`build_licenses.py`) a kontrola výkladů (`kontrola_vykladu.py`).

## Sestavení

    cargo test -p astro-core
    cargo build --release -p kosmogram --target x86_64-pc-windows-msvc

Instalátor a podepsaný balíček pro aktualizace vytvoří Tauri CLI 2 (`npm install @tauri-apps/cli@2`) ve složce `desktop`:

    $env:TAURI_SIGNING_PRIVATE_KEY = "<cesta k soukromému klíči>"
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<heslo>"
    tauri build --target x86_64-pc-windows-msvc

Výsledek je v `target/x86_64-pc-windows-msvc/release/bundle/nsis`. Při nové verzi se mění číslo v `core/Cargo.toml`, `desktop/Cargo.toml` a `desktop/tauri.conf.json`. Vydání vytváří `tools/release.ps1`.

## Licence

GNU GPL verze 2 nebo novější (`LICENSE`). Cizí části a jejich licence jsou v `NOTICE.md` a ve složce `licenses`.
