// Help centre: topics with search, inline figures, a comparison of house systems and on-screen tours.
// Content lives in help-content.js. Loaded after app.js, whose helpers ($, esc, invoke, normalize) it uses.

const TAB_TOPIC = { sky: "start", radix: "radix", transit: "tranzity", prog: "progrese", solar: "solar", calendar: "kalendar", rel: "vztah" };
const helpState = { topic: "start", query: "" };

// ---------- figures ----------
const fig = (viewBox, body, label, cls = "") => `<figure class="help-fig ${cls}"><svg viewBox="${viewBox}" role="img" aria-label="${esc(label)}">${body}</svg></figure>`;
const polar = (cx, cy, r, deg) => {
  const a = (deg * Math.PI) / 180;
  return [cx + r * Math.cos(a), cy + r * Math.sin(a)].map((n) => n.toFixed(1));
};
const ring = (cx, cy, r, attrs = "") => `<circle cx="${cx}" cy="${cy}" r="${r}" fill="none" stroke="var(--w-line)" ${attrs}/>`;
const spoke = (cx, cy, r1, r2, deg, attrs = 'stroke="var(--w-cusp)"') => {
  const [x1, y1] = polar(cx, cy, r1, deg), [x2, y2] = polar(cx, cy, r2, deg);
  return `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" ${attrs}/>`;
};
const num = (x, y, n) => `<circle cx="${x}" cy="${y}" r="11" fill="var(--accent)"/><text x="${x}" y="${y}" class="fig-num">${n}</text>`;
const zodiacBand = (cx, cy, rOut, rIn, size = 22, rot = 0) => {
  const fills = ["--w-fire", "--w-earth", "--w-air", "--w-water"], glyphs = "ABCDEFGHI\\KL";
  let o = "";
  for (let s = 0; s < 12; s++) {
    // Aries starts on the left and the signs run anticlockwise, as on the real wheel.
    const a = 180 + rot - s * 30, b = a - 30;
    const [x1, y1] = polar(cx, cy, rOut, a), [x2, y2] = polar(cx, cy, rOut, b), [x3, y3] = polar(cx, cy, rIn, b), [x4, y4] = polar(cx, cy, rIn, a);
    const [gx, gy] = polar(cx, cy, (rOut + rIn) / 2, a - 15);
    o += `<path d="M${x1} ${y1}A${rOut} ${rOut} 0 0 0 ${x2} ${y2}L${x3} ${y3}A${rIn} ${rIn} 0 0 1 ${x4} ${y4}Z" fill="var(${fills[s % 4]})" stroke="var(--w-line)"/><text x="${gx}" y="${gy}" class="fig-glyph" font-size="${size}">${esc(glyphs[s])}</text>`;
  }
  return o;
};
const body = (cx, cy, r, deg, glyph, color = "var(--w-ink)", size = 22) => {
  const [x, y] = polar(cx, cy, r, deg);
  return `<text x="${x}" y="${y}" class="fig-glyph" font-size="${size}" fill="${color}">${glyph}</text>`;
};
const swatch = (color, label) => `<span class="swatch"><i style="background:${color}"></i>${label}</span>`;

const HELP_FIGURES = {
  screen: () => {
    const box = (x, y, w, h, fill = "var(--card)") => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="6" fill="${fill}" stroke="var(--line)"/>`;
    const row = (x, y, w) => `<rect x="${x}" y="${y}" width="${w}" height="9" rx="3" fill="var(--line)"/>`;
    return fig("0 0 640 360", `
      ${box(0, 0, 640, 360, "var(--bg)")}${box(0, 0, 150, 360)}
      ${box(12, 40, 126, 24, "var(--accent-soft)")}${box(12, 72, 126, 24, "var(--accent)")}
      ${[112, 136, 160, 184, 208].map((y) => row(14, y, 100)).join("")}
      ${row(14, 312, 60)}${row(14, 332, 60)}
      ${[166, 236, 306, 376, 446, 516].map((x) => row(x, 70, 56)).join("")}
      <line x1="164" y1="88" x2="626" y2="88" stroke="var(--line)"/>
      ${box(164, 100, 250, 248)}${ring(289, 224, 108)}${ring(289, 224, 86)}${ring(289, 224, 62)}
      ${box(426, 100, 200, 248)}
      ${[126, 150, 174, 198, 222, 246, 270].map((y) => row(440, y, 170)).join("")}
      ${row(166, 20, 170)}${row(166, 40, 110)}
      ${num(138, 40, 1)}${num(138, 96, 2)}${num(138, 322, 3)}${num(372, 62, 4)}${num(289, 224, 5)}${num(626, 112, 6)}`, "Rozložení obrazovky s očíslovanými částmi", "wide");
  },
  wheel: () => {
    const c = 200;
    const lines = [[20, 140, "#2e7fb8"], [200, 290, "#c0392b"], [20, 200, "#c0392b"], [140, 200, "#2e9e6b"]];
    return fig("-24 -24 448 448", `
      ${zodiacBand(c, c, 190, 156)}${ring(c, c, 100)}
      ${[...Array(12)].map((_, i) => spoke(c, c, 100, 156, 180 - i * 30)).join("")}
      ${spoke(c, c, 100, 190, 180, 'stroke="var(--w-ink)" stroke-width="3"')}${spoke(c, c, 100, 190, 268, 'stroke="var(--w-ink)" stroke-width="3"')}
      ${[...Array(12)].map((_, i) => `<text x="${polar(c, c, 113, 165 - i * 30)[0]}" y="${polar(c, c, 113, 165 - i * 30)[1]}" class="fig-small">${i + 1}</text>`).join("")}
      ${lines.map(([a, b, color]) => { const [x1, y1] = polar(c, c, 100, a), [x2, y2] = polar(c, c, 100, b); return `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${color}" stroke-width="2"/>`; }).join("")}
      ${body(c, c, 136, 20, "Q")}${body(c, c, 136, 140, "R")}${body(c, c, 136, 200, "T")}${body(c, c, 136, 290, "U")}
      ${num(336, 44, 1)}${num(318, 214, 2)}${num(165, 330, 3)}${num(-6, 200, 4)}${num(192, -6, 5)}${num(190, 224, 6)}`, "Schéma kola s očíslovanými částmi");
  },
  elements: () => `<p class="swatches">${swatch("var(--w-fire)", "Oheň: Beran, Lev, Střelec")}${swatch("var(--w-earth)", "Země: Býk, Panna, Kozoroh")}${swatch("var(--w-air)", "Vzduch: Blíženci, Váhy, Vodnář")}${swatch("var(--w-water)", "Voda: Rak, Štír, Ryby")}</p>`,
  aspectColors: () => `<p class="swatches">${swatch("#c98a12", "Konjunkce")}${swatch("#c0392b", "Opozice a kvadratura")}${swatch("#2e7fb8", "Trigon")}${swatch("#2e9e6b", "Sextil")}${swatch("#8a6db8", "Vedlejší aspekty")}</p>`,
  glyphs: () => {
    const list = (items) => items.map(([g, n]) => `<span><span class="gl">${esc(g)}</span> ${n}</span>`).join("");
    return `<div class="glyphs">${list([["Q", "Slunce"], ["R", "Měsíc"], ["S", "Merkur"], ["T", "Venuše"], ["U", "Mars"], ["V", "Jupiter"], ["W", "Saturn"], ["X", "Uran"], ["Y", "Neptun"], ["Z", "Pluto"], ["q", "Chiron"], ["g", "Severní uzel"], ["c", "Ascendent"], ["d", "Medium coeli"]])}</div>
      <div class="glyphs">${list([["A", "Beran"], ["B", "Býk"], ["C", "Blíženci"], ["D", "Rak"], ["E", "Lev"], ["F", "Panna"], ["G", "Váhy"], ["H", "Štír"], ["I", "Střelec"], ["\\", "Kozoroh"], ["K", "Vodnář"], ["L", "Ryby"]])}</div>
      <div class="glyphs">${list([["!", "Konjunkce"], ['"', "Opozice"], ["#", "Kvadratura"], ["$", "Trigon"], ["%", "Sextil"], ["&", "Kvinkunx"], ["'", "Polosextil"], ["(", "Polokvadrát"], [")", "Seskvikvadrát"]])}</div>`;
  },
  double: () => {
    const c = 150;
    const [x1, y1] = polar(c, c, 66, 60), [x2, y2] = polar(c, c, 66, 200);
    return fig("0 0 480 300", `
      ${zodiacBand(c, c, 144, 120, 16)}${ring(c, c, 88)}${ring(c, c, 66)}
      <line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="#2e7fb8" stroke-width="2"/>
      ${body(c, c, 104, 200, "V", "var(--w-second)", 20)}${body(c, c, 76, 60, "Q", "var(--w-ink)", 18)}${body(c, c, 76, 140, "R", "var(--w-ink)", 18)}${body(c, c, 104, 300, "W", "var(--w-second)", 20)}
      <text x="316" y="106" class="fig-label" fill="var(--w-second)">Vnější prstenec</text>
      <text x="316" y="128" class="fig-label">tranzity, progrese</text><text x="316" y="150" class="fig-label">nebo druhý člověk</text>
      <text x="316" y="192" class="fig-label" fill="var(--w-ink)">Vnitřní kolo</text><text x="316" y="214" class="fig-label">radix</text>`, "Vnitřní kolo a vnější prstenec", "wide");
  },
  dayYear: () => {
    const cell = (x, top, bottom, on) => `<rect x="${x}" y="26" width="72" height="44" rx="6" fill="${on ? "var(--accent-soft)" : "var(--card)"}" stroke="var(--line)"/><text x="${x + 36}" y="50" class="fig-label center">${top}</text><text x="${x + 36}" y="96" class="fig-label center">${bottom}</text><line x1="${x + 36}" y1="70" x2="${x + 36}" y2="82" stroke="var(--accent)"/>`;
    return fig("0 0 480 106", `<text x="4" y="16" class="fig-label">Dny po narození a roky života</text>${cell(0, "narození", "věk 0", true)}${cell(84, "2. den", "věk 1 rok")}${cell(168, "3. den", "věk 2 roky")}${cell(252, "31. den", "věk 30 let")}${cell(336, "61. den", "věk 60 let")}`, "Jeden den po narození odpovídá jednomu roku", "wide");
  },
  composite: () => {
    const c = 110, a = 200, b = 300, m = 250;
    return fig("0 0 480 220", `${ring(c, c, 90)}
      ${spoke(c, c, 0, 90, a, 'stroke="#2e7fb8" stroke-width="2"')}${spoke(c, c, 0, 90, b, 'stroke="#7a3fa0" stroke-width="2"')}${spoke(c, c, 0, 100, m, 'stroke="var(--accent)" stroke-width="3" stroke-dasharray="6 4"')}
      <text x="230" y="70" class="fig-label" fill="#2e7fb8">Venuše prvního člověka</text>
      <text x="230" y="100" class="fig-label" fill="#7a3fa0">Venuše druhého člověka</text>
      <text x="230" y="130" class="fig-label" fill="var(--accent)">Složená Venuše: střed mezi nimi</text>`, "Složená poloha je středem mezi dvěma polohami", "wide");
  },
  houseKinds: () => {
    const wheel = (cx, label, cusps, rot = 0) => `${zodiacBand(cx, 90, 82, 66, 12, rot)}${ring(cx, 90, 66)}${cusps.map((d) => spoke(cx, 90, 24, 66, d, 'stroke="var(--accent)"')).join("")}${ring(cx, 90, 24)}${spoke(cx, 90, 24, 82, 180, 'stroke="var(--w-ink)" stroke-width="2.5"')}<text x="${cx}" y="192" class="fig-label center">${label}</text>`;
    const even = [...Array(12)].map((_, i) => 180 - i * 30);
    const shift = 12; // The ascendant lies inside its sign, so whole-sign cusps miss it.
    const uneven = [180, 142, 112, 86, 58, 26, 0, 322, 292, 266, 238, 206];
    return fig("0 0 540 200", wheel(90, "Rovné domy", even, shift) + wheel(270, "Celá znamení", even.map((d) => d + shift), shift) + wheel(450, "Placidus a podobné", uneven, shift), "Tři způsoby dělení na domy", "wide");
  },
  aspectAngles: () => {
    const mini = (x, y, angle, name, color) => {
      const [x2, y2] = polar(x, y, 34, 180 - angle);
      const [x1, y1] = polar(x, y, 34, 180);
      return `${ring(x, y, 34)}<line x1="${x}" y1="${y}" x2="${x1}" y2="${y1}" stroke="${color}" stroke-width="2.5"/><line x1="${x}" y1="${y}" x2="${x2}" y2="${y2}" stroke="${color}" stroke-width="2.5"/><text x="${x}" y="${y + 52}" class="fig-small">${name} ${angle}°</text>`;
    };
    return fig("0 0 540 106", mini(50, 40, 0, "Konjunkce", "#c98a12") + mini(158, 40, 60, "Sextil", "#2e9e6b") + mini(266, 40, 90, "Kvadratura", "#c0392b") + mini(374, 40, 120, "Trigon", "#2e7fb8") + mini(482, 40, 180, "Opozice", "#c0392b"), "Pět hlavních aspektů jako úhly", "wide");
  },
  axes: () => {
    const c = 72;
    const arrow = (cx, label) => `${ring(cx, c, 58)}${spoke(cx, c, 0, 58, 180, 'stroke="var(--w-ink)" stroke-width="2"')}<text x="${cx}" y="${c * 2 + 14}" class="fig-label center">${label}</text>`;
    const full = `<path d="M${polar(c, c, 66, 190).join(" ")}A66 66 0 1 1 ${polar(c, c, 66, 170).join(" ")}" fill="none" stroke="var(--accent)" stroke-width="3"/>`;
    const small = `<path d="M${polar(360, c, 66, 180).join(" ")}A66 66 0 0 1 ${polar(360, c, 66, 186).join(" ")}" fill="none" stroke="var(--accent)" stroke-width="5"/>`;
    return fig("0 0 480 170", arrow(c, "Astrolog: za rok celý kruh") + full + arrow(360, "Solární oblouk: asi 1° za rok") + small, "Pohyb os za jeden rok oběma způsoby", "wide");
  },
};

// ---------- centre ----------
const plain = (t) => t.html.replace(/\{\{\w+\}\}/g, " ").replace(/<[^>]+>/g, " ");
const haystack = new Map();
function matches(t, query) {
  if (!haystack.has(t.id)) haystack.set(t.id, normalize(`${t.title} ${t.summary} ${t.keys} ${plain(t)}`));
  return query.split(/\s+/).filter(Boolean).every((w) => haystack.get(t.id).includes(w));
}

function renderHelpNav() {
  const q = normalize(helpState.query.trim());
  const found = HELP_TOPICS.filter((t) => matches(t, q));
  $("help-nav").innerHTML = found.length
    ? HELP_AREAS.map(([id, label]) => {
        const items = found.filter((t) => t.area === id);
        return items.length ? `<h3>${label}</h3><ul>${items.map((t) => `<li><button type="button" data-topic="${t.id}" class="${t.id === helpState.topic ? "on" : ""}">${esc(t.title)}</button></li>`).join("")}</ul>` : "";
      }).join("")
    : `<p class="muted">Nic takového v nápovědě není. Zkuste jiné slovo.</p>`;
}

function showTopic(id) {
  const t = HELP_TOPICS.find((x) => x.id === id) ?? HELP_TOPICS[0];
  helpState.topic = t.id;
  const html = t.html.replace(/\{\{(\w+)\}\}/g, (_, name) => HELP_FIGURES[name]()).replace(/<a data-topic/g, '<a href="#" data-topic');
  $("help-article").innerHTML = `<button type="button" id="help-back">← Všechna témata</button><h2>${esc(t.title)}</h2><p class="help-summary">${esc(t.summary)}</p>${t.tour ? `<p><button type="button" id="help-tour" class="primary">Ukázat v programu</button></p>` : ""}${t.compare ? compareBox() : ""}${html}`;
  $("help-article").scrollTop = 0;
  $("help-dialog").classList.add("reading");
  renderHelpNav();
}

function openHelp(id) {
  if (!$("help-dialog").open) {
    helpState.query = "";
    $("help-search").value = "";
    $("help-dialog").showModal();
    $("help-search").focus();
  }
  showTopic(id ?? TAB_TOPIC[state.tab] ?? "start");
}

$("open-help").onclick = () => openHelp();
$("help-close").onclick = () => $("help-dialog").close();
$("help-search").addEventListener("input", (e) => {
  helpState.query = e.target.value;
  $("help-dialog").classList.remove("reading");
  renderHelpNav();
});

// One listener for topic links, the "?" buttons in forms and buttons inside an article.
document.addEventListener("click", (e) => {
  const link = e.target.closest("[data-topic]");
  if (link) {
    e.preventDefault();
    return openHelp(link.dataset.topic);
  }
  if (e.target.id === "help-back") $("help-dialog").classList.remove("reading");
  if (e.target.id === "help-tour") startTour(HELP_TOPICS.find((t) => t.id === helpState.topic).tour);
  if (e.target.id === "help-compare") compareHouses();
});

document.addEventListener("keydown", (e) => {
  if (e.key !== "F1") return;
  e.preventDefault();
  if (!tour) openHelp();
});

// ---------- comparison of house systems ----------
const ANGLE_GLYPHS = ["c", "d"]; // Ascendant and MC lie on the cusps, so their house says nothing.

function compareBox() {
  const p = state.current;
  const ready = p?.time_known;
  return `<div class="compare"><b>Porovnání na konkrétním horoskopu</b>
    <p>${ready ? `Spočítá domy pro člověka ${esc(p.name)} ve všech osmi systémech a ukáže, které planety v jiném systému leží v jiném domě.` : "Vyberte v seznamu člověka se známým časem narození. Bez času domy nejsou."}</p>
    ${ready ? `<button type="button" id="help-compare">Porovnat systémy domů</button>` : ""}<div id="compare-result"></div></div>`;
}

async function compareHouses() {
  const box = $("compare-result");
  box.innerHTML = `<p class="muted">Počítám…</p>`;
  const systems = await invoke("house_systems");
  const results = await Promise.all(systems.map(async ([key, name]) => {
    try {
      const panel = await invoke("radix", { person: state.current, settings: { ...state.settings, house_system: key } });
      return { key, name, rows: panel.rows };
    } catch {
      return { key, name, rows: null };
    }
  }));
  const usable = results.filter((r) => r.rows);
  const rows = (usable[0]?.rows ?? []).filter((r) => r.house && !ANGLE_GLYPHS.includes(r.glyph));
  const head = `<tr><th>Těleso</th>${results.map((r) => `<th class="${r.key === state.settings.house_system ? "cur" : ""}">${esc(r.name)}</th>`).join("")}</tr>`;
  const body = rows.map((row, i) => {
    const houses = usable.map((r) => r.rows.find((x) => x.name === row.name).house);
    const differs = new Set(houses).size > 1;
    return `<tr class="${differs ? "diff" : ""}"><td><span class="gl">${esc(row.glyph)}</span> ${esc(row.name)}</td>${results.map((r) => `<td class="${r.key === state.settings.house_system ? "cur" : ""}">${r.rows ? r.rows.find((x) => x.name === row.name).house : "–"}</td>`).join("")}</tr>`;
  }).join("");
  const changed = rows.filter((row) => new Set(usable.map((r) => r.rows.find((x) => x.name === row.name).house)).size > 1).length;
  box.innerHTML = `<div class="compare-scroll"><table>${head}${body}</table></div><p class="note">Čísla jsou domy. ${changed ? `Zvýrazněná tělesa mění dům podle systému (${changed} z ${rows.length}).` : "Všechna tělesa zůstávají ve stejném domě."} Sloupec aktuálního systému je podbarvený. Nic se tím nepřepne, nastavení zůstává.</p>`;
}

// ---------- tour ----------
let tour = null;
const helpEl = (name) => document.querySelector(`[data-help="${name}"]`);
const visible = (el) => !!el && el.getClientRects().length > 0 && el.getBoundingClientRect().width > 0;

async function waitFor(name) {
  for (let i = 0; i < 40; i++) {
    if (visible(helpEl(name))) return helpEl(name);
    await new Promise((r) => setTimeout(r, 50));
  }
  return null;
}

function goTo(step) {
  if (step.tab && state.tab !== step.tab) document.querySelector(`#tabs button[data-tab="${step.tab}"]`)?.click();
  if (step.sub != null) document.querySelector(`#subtabs button[data-i="${step.sub}"]`)?.click();
}

function placeTour() {
  if (!tour) return;
  const r = tour.el.getBoundingClientRect(), pad = 6, ring = $("tour-ring"), card = $("tour-card");
  // A target taller than the window is shown from its top, so the card still has room.
  const top = Math.max(4, r.top - pad), left = Math.max(4, r.left - pad);
  const bottom = Math.min(innerHeight - 4, r.bottom + pad), right = Math.min(innerWidth - 4, r.right + pad);
  Object.assign(ring.style, { top: `${top}px`, left: `${left}px`, width: `${right - left}px`, height: `${bottom - top}px` });
  const c = card.getBoundingClientRect(), gap = 12;
  let y = bottom + gap;
  if (y + c.height > innerHeight - 8) y = top - c.height - gap;
  let x = left;
  if (y < 8) {
    // No room above or below: beside the target, or over it when it fills the window.
    y = Math.min(top + 8, innerHeight - c.height - 8);
    x = right + gap + c.width < innerWidth ? right + gap : left - gap - c.width >= 8 ? left - gap - c.width : innerWidth - c.width - 12;
  }
  card.style.top = `${Math.max(8, y)}px`;
  card.style.left = `${Math.max(8, Math.min(x, innerWidth - c.width - 8))}px`;
}

async function showStep(i, dir = 1) {
  const run = tour;
  if (!run) return;
  if (i < 0) return;
  if (i >= run.steps.length) return endTour();
  const step = run.steps[i];
  goTo(step);
  const el = await waitFor(step.at);
  if (tour !== run) return;
  if (!el) return showStep(i + dir, dir);
  run.i = i;
  run.el = el;
  el.scrollIntoView({ block: "nearest" });
  $("tour-title").textContent = step.title;
  $("tour-text").textContent = step.text;
  $("tour-count").textContent = `${run.steps.indexOf(step) + 1} z ${run.steps.length}`;
  $("tour-prev").disabled = i === 0;
  $("tour-next").textContent = i === run.steps.length - 1 ? "Hotovo" : "Další";
  placeTour();
  $("tour-next").focus();
}

async function startTour(name) {
  // The form keeps its choices only until closed, so a tour started from it must not lose them.
  if ($("settings-dialog").open) $("settings-save").click();
  document.querySelectorAll("dialog[open]").forEach((d) => d.close());
  const back = state.current?.id;
  const needsPerson = name !== "sky" && name !== "start";
  if (name === "sky" && state.tab !== "sky") showSky();
  if (name !== "sky" && !state.current && state.people[0]) selectPerson(state.people[0].id);
  if (needsPerson && !state.current) return openHelpNotice();
  const steps = HELP_TOURS[name].filter((s) => state.current || !s.tab);
  tour = { steps, i: 0, el: null, back: name === "sky" ? back : null };
  $("tour").hidden = false;
  showStep(0);
}

function openHelpNotice() {
  openHelp(helpState.topic);
  $("help-article").insertAdjacentHTML("afterbegin", `<p class="tip">${SKIP_WITHOUT_PERSON}</p>`);
}

function endTour() {
  const back = tour?.back;
  tour = null;
  $("tour").hidden = true;
  if (back) selectPerson(back);
}

$("tour-next").onclick = () => tour && showStep(tour.i + 1);
$("tour-prev").onclick = () => tour && showStep(tour.i - 1, -1);
$("tour-end").onclick = endTour;
addEventListener("resize", placeTour);
document.addEventListener("keydown", (e) => {
  if (!tour) return;
  if (e.key === "Escape") endTour();
  if (e.key === "ArrowRight") showStep(tour.i + 1);
  if (e.key === "ArrowLeft") showStep(tour.i - 1, -1);
  if (["Escape", "ArrowLeft", "ArrowRight"].includes(e.key)) { e.preventDefault(); e.stopPropagation(); }
}, true);
