const invoke = window.__TAURI__.core.invoke;
const $ = (id) => document.getElementById(id);
const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
const MONTHS = ["Celý rok", "leden", "únor", "březen", "duben", "květen", "červen", "červenec", "srpen", "září", "říjen", "listopad", "prosinec"];

const state = { people: [], temps: [], settings: null, current: null, tab: "radix", sub: 0, panel: null, partnerId: "", relView: "synastry", relation: null, editing: null, place: null };
const today = new Date();
const pad = (n) => String(n).padStart(2, "0");
const isoDate = (d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
const when = { date: isoDate(today), time: `${pad(today.getHours())}:${pad(today.getMinutes())}`, year: today.getFullYear(), month: today.getMonth() + 1, progDate: isoDate(today), important: true };
const solar = { year: today.getFullYear(), place: null };
// `from` null means the lunar return in effect today; `date` is the shown return, for stepping.
const lunar = { from: null, date: null };
const addDays = (iso, n) => { const d = new Date(`${iso}T12:00:00Z`); d.setUTCDate(d.getUTCDate() + n); return d.toISOString().slice(0, 10); };

const allPeople = () => [...state.temps, ...state.people];
const setTemps = (temps) => {
  state.temps = temps;
  invoke("set_unsaved", { unsaved: temps.length > 0 });
};

function showError(msg) {
  $("error").hidden = !msg;
  $("error").textContent = msg ? String(msg) : "";
}

// ---------- people list ----------
function czDate(iso) {
  const [y, m, d] = iso.split("-").map(Number);
  return `${d}. ${m}. ${y}`;
}

const NO_GROUP = "-bez-skupiny-";
const groupOf = (p) => (p.group ?? "").trim();
// Group names as people wrote them, alphabetically; "Rodina" and "rodina" count as one.
function groupNames() {
  const seen = new Map();
  for (const p of allPeople()) {
    const g = groupOf(p);
    if (g && !seen.has(g.toLocaleLowerCase("cs"))) seen.set(g.toLocaleLowerCase("cs"), g);
  }
  return [...seen.values()].sort((a, b) => a.localeCompare(b, "cs"));
}
const sameGroup = (a, b) => a.toLocaleLowerCase("cs") === b.toLocaleLowerCase("cs");
// A new person gets the group shown in the list, so adding family while looking at family needs no typing.
const only = () => ($("group-filter").value === NO_GROUP ? "" : $("group-filter").value);
// "rodina " is saved as the existing "Rodina", so one group does not split in two.
function canonicalGroup(text) {
  const g = text.trim().replace(/\s+/g, " ");
  return groupNames().find((n) => sameGroup(n, g)) ?? g;
}

function renderGroupFilter() {
  const sel = $("group-filter");
  const names = groupNames();
  const chosen = sel.value;
  sel.hidden = names.length === 0;
  sel.innerHTML = `<option value="">Všechny skupiny</option>${names.map((g) => `<option value="${esc(g)}">${esc(g)}</option>`).join("")}<option value="${NO_GROUP}">Bez skupiny</option>`;
  sel.value = chosen === NO_GROUP || names.some((g) => g === chosen) ? chosen : "";
  $("group-list").innerHTML = names.map((g) => `<option value="${esc(g)}">`).join("");
}

function renderPeople() {
  renderGroupFilter();
  const q = normalize($("filter").value);
  const only = $("group-filter").value;
  const item = (p, cls = "") => `<li class="${cls}"><button data-id="${esc(p.id)}" class="${state.current?.id === p.id ? "on" : ""}">${esc(p.name)}<small>${esc(czDate(p.date))}, ${esc(p.place.name)}</small></button></li>`;
  const inGroup = (p) => !only || (only === NO_GROUP ? !groupOf(p) : sameGroup(groupOf(p), only));
  const found = (p) => inGroup(p) && normalize(`${p.name} ${placeLabel(p.place)} ${p.note ?? ""} ${groupOf(p)}`).includes(q);
  const temps = state.temps.filter(found);
  const saved = state.people.filter(found);
  let html = temps.length ? `<li class="group">Neuloženo, po zavření zmizí</li>${temps.map((p) => item(p)).join("")}` : "";
  const names = groupNames();
  if (only || !names.length) {
    html += saved.map((p, i) => item(p, temps.length && i === 0 ? "saved-start" : "")).join("");
  } else {
    // All groups at once: a heading for each, people without a group at the end.
    const sections = [...names.map((g) => [g, saved.filter((p) => sameGroup(groupOf(p), g))]), ["Bez skupiny", saved.filter((p) => !groupOf(p))]];
    for (const [title, list] of sections.filter(([, l]) => l.length)) {
      html += `<li class="group">${esc(title)}</li>${list.map((p) => item(p)).join("")}`;
    }
  }
  $("people").innerHTML = html;
}

function normalize(s) {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
}

$("people").addEventListener("click", (e) => {
  const b = e.target.closest("button");
  if (b) selectPerson(b.dataset.id);
});
$("filter").addEventListener("input", renderPeople);
$("group-filter").addEventListener("change", renderPeople);
$("f-name").addEventListener("input", showNameHint);
$("f-sex").addEventListener("change", showNameHint);

function selectPerson(id) {
  state.current = allPeople().find((p) => p.id === id) || null;
  state.relation = null;
  state.partnerId = "";
  solar.place = null;
  lunar.from = null;
  if (!state.current) return showSky();
  if (state.tab === "sky") {
    state.tab = "radix";
    state.sub = 0;
    document.querySelectorAll("#tabs button").forEach((x) => x.classList.toggle("on", x.dataset.tab === "radix"));
  }
  $("open-sky").classList.remove("on");
  $("sky-bar").hidden = true;
  $("tabs").hidden = false;
  $("edit-person").hidden = false;
  $("save-temp").hidden = !state.current.temp;
  renderPeople();
  const p = state.current;
  $("p-name").textContent = p.name;
  $("p-info").textContent = `${czDate(p.date)}${p.time_known ? ", " + p.time : ", čas neznámý"}, ${p.place.name}`;
  $("p-note").textContent = p.note ?? "";
  $("p-note").hidden = !p.note;
  refresh();
}

// ---------- tabs ----------
$("tabs").addEventListener("click", (e) => {
  const b = e.target.closest("button");
  if (!b) return;
  state.tab = b.dataset.tab;
  state.sub = 0;
  document.querySelectorAll("#tabs button").forEach((x) => x.classList.toggle("on", x === b));
  refresh();
});

let refreshId = 0;
/** Shows how the name will read in relationship texts, so a wrong guess is easy to spot. */
async function showNameHint() {
  const name = $("f-name").value.trim();
  const hint = $("f-name-hint");
  hint.hidden = !name;
  if (!name) return;
  const n = await invoke("name_forms", { name, sex: $("f-sex").value });
  if (name !== $("f-name").value.trim()) return;
  hint.textContent = `Ve výkladech: ${n.cases[0]}, Slunce ${n.cases[1]}, s ${n.cases[5]} (${n.female ? "žena" : "muž"})`;
}

async function refresh() {
  const id = ++refreshId;
  renderControls();
  renderSolarBar();
  showError("");
  $("calendar").hidden = state.tab !== "calendar";
  const p = state.current, settings = state.settings;
  try {
    let panel;
    if (state.tab === "sky") {
      renderSkyBar();
      if (!sky.place) return showPanel(null);
      panel = await invoke("sky", { place: sky.place, date: sky.date, time: sky.time, settings });
    }
    if (state.tab === "radix") panel = await invoke("radix", { person: p, settings });
    if (state.tab === "transit") panel = await invoke("transits", { person: p, date: when.date, time: when.time, settings });
    if (state.tab === "prog") panel = await invoke("progressions", { person: p, date: when.progDate, settings });
    if (state.tab === "solar") panel = await invoke("solar_return", { person: p, year: solar.year, place: solar.place, settings });
    if (state.tab === "lunar") panel = await lunarReturn(p, settings);
    if (state.tab === "calendar") {
      showPanel(null);
      return loadCalendar();
    }
    if (state.tab === "rel") {
      const partner = allPeople().find((x) => x.id === state.partnerId);
      if (!partner) return showPanel(null);
      state.relation = await invoke("relation", { a: p, b: partner, settings });
      panel = state.relation[state.relView];
    }
    if (id === refreshId) showPanel(panel);
  } catch (e) {
    if (id === refreshId) { showPanel(null); showError(e); }
  }
}

function field(label, html) {
  return `<label>${label}${html}</label>`;
}

function renderControls() {
  const c = $("controls");
  if (["sky", "radix", "solar", "lunar"].includes(state.tab)) c.innerHTML = "";
  if (state.tab === "calendar") {
    c.innerHTML = field("Období", `<select id="c-month">${MONTHS.map((m, i) => `<option value="${i}" ${i === when.month ? "selected" : ""}>${m}</option>`).join("")}</select>`) +
      field("Rok", `<input type="number" id="c-year" value="${when.year}" min="1800" max="2399">`) +
      `<label class="check"><input type="checkbox" id="c-important" ${when.important ? "checked" : ""}> Jen důležité (Jupiter a pomalejší)</label>`;
  }
  if (state.tab === "transit") {
    c.innerHTML = field("Den", `<input type="date" id="c-date" value="${when.date}">`) + field("Čas", `<input type="time" id="c-time" value="${when.time}">`) + `<button id="c-now">Teď</button>`;
  }
  if (state.tab === "prog") c.innerHTML = field("Den", `<input type="date" id="c-pdate" value="${when.progDate}">`) + `<button id="c-pnow">Dnes</button>`;
  if (state.tab === "rel") {
    const others = allPeople().filter((p) => p.id !== state.current.id);
    c.innerHTML = field("Druhý člověk", `<select id="c-partner"><option value="">Vyberte…</option>${others.map((p) => `<option value="${esc(p.id)}" ${p.id === state.partnerId ? "selected" : ""}>${esc(p.name)}</option>`).join("")}</select>`) +
      field("Zobrazení", `<select id="c-rel"><option value="synastry" ${state.relView === "synastry" ? "selected" : ""}>Dvě kola přes sebe</option><option value="composite" ${state.relView === "composite" ? "selected" : ""}>Složený horoskop</option></select>`) +
      (state.partnerId ? "" : `<p class="muted">${others.length ? "Vyberte druhého člověka." : "Nejdřív přidejte druhého člověka tlačítkem Nový člověk."}</p>`);
  }
}

$("controls").addEventListener("change", (e) => {
  const t = e.target;
  if (t.id === "c-date" && t.value) when.date = t.value;
  if (t.id === "c-time" && t.value) when.time = t.value;
  if (t.id === "c-pdate" && t.value) when.progDate = t.value;
  if (t.id === "c-partner") state.partnerId = t.value;
  if (t.id === "c-rel") state.relView = t.value;
  if (t.id === "c-year") when.year = Number(t.value);
  if (t.id === "c-month") when.month = Number(t.value);
  if (t.id === "c-important") when.important = t.checked;
  if (state.tab === "calendar") return loadCalendar();
  if (["c-year", "c-month"].includes(t.id)) return loadEvents();
  refresh();
});

$("controls").addEventListener("click", (e) => {
  const now = new Date();
  if (e.target.id === "c-now") { when.date = isoDate(now); when.time = `${pad(now.getHours())}:${pad(now.getMinutes())}`; refresh(); }
  if (e.target.id === "c-pnow") { when.progDate = isoDate(now); refresh(); }
});

// ---------- result panel ----------
function subTabs() {
  if (state.tab === "sky") return ["Planety", "Aspekty"];
  const list = ["Planety", "Aspekty", "Výklad"];
  if (state.tab === "prog") list.push("Kalendář");
  return list;
}

function showPanel(panel) {
  state.panel = panel;
  $("view").hidden = !panel;
  if (!panel) return;
  $("wheel").innerHTML = panel.svg;
  const subs = subTabs();
  if (state.sub >= subs.length) state.sub = 0;
  $("subtabs").innerHTML = subs.map((s, i) => `<button data-i="${i}" class="${i === state.sub ? "on" : ""}">${s}</button>`).join("");
  renderSub();
}

$("subtabs").addEventListener("click", (e) => {
  const b = e.target.closest("button");
  if (!b) return;
  state.sub = Number(b.dataset.i);
  document.querySelectorAll("#subtabs button").forEach((x) => x.classList.toggle("on", x === b));
  renderSub();
});

function renderSub() {
  const name = subTabs()[state.sub];
  const p = state.panel;
  const panel = $("panel");
  if (name === "Planety") panel.innerHTML = planetsTable(p);
  if (name === "Aspekty") panel.innerHTML = aspectsTable(p);
  if (name === "Výklad") panel.innerHTML = sectionsHtml(p);
  if (state.tab !== "calendar") document.querySelectorAll("#c-month, #c-year").forEach((x) => { if (name !== "Kalendář") x.closest("label").remove(); });
  if (name === "Kalendář") loadEvents();
}

function planetsTable(p) {
  const hasHouse = p.rows.some((r) => r.house);
  const body = p.rows.map((r) => `<tr><td><span class="gl">${esc(r.glyph)}</span> ${esc(r.name)}${r.retro ? ' <span class="muted" title="Zpětný pohyb">R</span>' : ""}</td><td>${esc(r.position)}</td>${hasHouse ? `<td>${r.house ?? ""}</td>` : ""}<td class="muted">${esc(r.dignity)}</td></tr>`).join("");
  const cusps = p.cusps.length ? `<h3>Domy</h3><table><tr><th>Dům</th><th>Hrot</th><th>Dům</th><th>Hrot</th></tr>${p.cusps.slice(0, 6).map((c, i) => `<tr><td>${c.house}.</td><td>${esc(c.position)}</td><td>${p.cusps[i + 6].house}.</td><td>${esc(p.cusps[i + 6].position)}</td></tr>`).join("")}</table>` : "";
  return `<table><tr><th>Planeta</th><th>Poloha</th>${hasHouse ? "<th>Dům</th>" : ""}<th></th></tr>${body}</table>${cusps}${p.info.map((t) => `<p class="note">${esc(t)}</p>`).join("")}`;
}

function aspectsTable(p) {
  if (!p.aspects.length) return `<p class="muted">Žádné aspekty v rozsahu.</p>`;
  return `<table><tr><th>Aspekt</th><th>Orb</th><th></th></tr>${p.aspects.map((a) => `<tr><td><span class="gl">${esc(a.a_glyph)}</span> ${esc(a.a_name)} <span class="gl" style="color:${a.color}" title="${esc(a.aspect_name)}">${esc(a.glyph)}</span> <span class="gl">${esc(a.b_glyph)}</span> ${esc(a.b_name)}<br><small class="muted">${esc(a.aspect_name)}</small></td><td>${a.orb.toFixed(1)}°</td><td class="muted">${a.applying ? "sbíhá se" : "rozbíhá se"}</td></tr>`).join("")}</table>`;
}

function sectionsHtml(p) {
  return p.sections.map((s) => `<h3>${esc(s.title)}</h3>${s.items.length ? s.items.map((i) => `<div class="item"><b>${esc(i.title)}</b>${esc(i.text)}</div>`).join("") : '<p class="muted">Nic v tomto období.</p>'}`).join("");
}

let eventsId = 0;
async function loadEvents() {
  const id = ++eventsId;
  const c = $("controls");
  if (!$("c-month")) {
    c.insertAdjacentHTML("beforeend", field("Měsíc", `<select id="c-month">${MONTHS.map((m, i) => `<option value="${i}" ${i === when.month ? "selected" : ""}>${m}</option>`).join("")}</select>`) + field("Rok", `<input type="number" id="c-year" value="${when.year}" min="1800" max="2399">`));
  }
  const panel = $("panel");
  panel.innerHTML = `<p class="muted">Počítám…</p>`;
  try {
    const list = await invoke("events", { person: state.current, year: when.year, month: when.month, progression: state.tab === "prog", settings: state.settings });
    if (id !== eventsId || subTabs()[state.sub] !== "Kalendář") return;
    panel.innerHTML = list.length ? list.map((e) => `<div class="item"><span class="event-date">${esc(e.date)} ${esc(e.time)}</span><b><span class="dot" style="background:${e.hit.color}"></span>${esc(e.title)}</b>${esc(e.text)}</div>`).join("") : `<p class="muted">V tomto období nic přesného nenastává.</p>`;
  } catch (e) {
    panel.innerHTML = "";
    showError(e);
  }
}

// ---------- calendar of transits ----------
const FIRST_SLOW_BODY = 5; // Jupiter; everything faster passes too often to stand out

function eventHtml(e) {
  return `<div class="item"><span class="event-date">${esc(e.date)} ${esc(e.time)}</span><b><span class="dot" style="background:${e.hit.color}"></span>${esc(e.title)}</b>${esc(e.text)}</div>`;
}

function calendarHtml(list, total) {
  if (!list.length) return `<p class="muted">${total ? "Žádná důležitá událost. Zrušte zaškrtnutí Jen důležité." : "V tomto období nic přesného nenastává."}</p>`;
  const groups = new Map();
  for (const e of list) {
    const month = Number(e.date.split(". ")[1]);
    if (!groups.has(month)) groups.set(month, []);
    groups.get(month).push(e);
  }
  return [...groups].map(([month, items]) => `<h3>${MONTHS[month]} ${when.year}</h3>${items.map(eventHtml).join("")}`).join("");
}

let calendarId = 0;
async function loadCalendar() {
  const id = ++calendarId;
  const box = $("calendar");
  state.calendar = null;
  box.innerHTML = `<p class="muted">Počítám…</p>`;
  try {
    const all = await invoke("events", { person: state.current, year: when.year, month: when.month, progression: false, settings: state.settings });
    if (id !== calendarId || state.tab !== "calendar") return;
    const list = when.important ? all.filter((e) => e.hit.mover >= FIRST_SLOW_BODY) : all;
    const label = `${MONTHS[when.month] === MONTHS[0] ? "rok" : MONTHS[when.month]} ${when.year}`;
    state.calendar = { title: `Kalendář tranzitů: ${state.current.name}`, label: `${label}${when.important ? ", jen důležité" : ""}`, html: calendarHtml(list, all.length) };
    box.innerHTML = state.calendar.html;
  } catch (e) {
    if (id !== calendarId) return;
    box.innerHTML = "";
    showError(e);
  }
}

// ---------- solar and lunar return ----------
function renderSolarBar() {
  const lunarTab = state.tab === "lunar";
  $("solar-bar").hidden = state.tab !== "solar" && !lunarTab;
  if ($("solar-bar").hidden) return;
  $("solar-year-label").hidden = lunarTab;
  $("lunar-step").hidden = !lunarTab;
  $("lunar-now").disabled = lunar.from === null;
  $("solar-year").value = solar.year;
  $("solar-place").value = placeLabel(solar.place ?? state.current.place);
  $("solar-home").hidden = !solar.place;
}

$("solar-year").onchange = (e) => {
  const year = Number(e.target.value);
  if (year >= 1800 && year <= 2399) { solar.year = year; refresh(); } else renderSolarBar();
};
$("solar-home").onclick = () => { solar.place = null; refresh(); };

/** The return asked for; without one, the newest return that is not in the future. */
async function lunarReturn(p, settings) {
  const now = isoDate(today);
  let [panel, date] = await invoke("lunar_return", { person: p, from: lunar.from ?? addDays(now, -28), place: solar.place, settings });
  if (lunar.from === null && date < now) {
    const [next, nextDate] = await invoke("lunar_return", { person: p, from: addDays(date, 1), place: solar.place, settings });
    if (nextDate <= now) [panel, date] = [next, nextDate];
  }
  lunar.date = date;
  return panel;
}
$("lunar-prev").onclick = () => { if (lunar.date) { lunar.from = addDays(lunar.date, -28); refresh(); } };
$("lunar-next").onclick = () => { if (lunar.date) { lunar.from = addDays(lunar.date, 1); refresh(); } };
$("lunar-now").onclick = () => { lunar.from = null; refresh(); };

// ---------- person form ----------
function parseCzDate(s) {
  const m = s.trim().match(/^(\d{1,2})[.\s/-]+(\d{1,2})[.\s/-]+(\d{4})$/) ;
  if (m) return `${m[3]}-${pad(m[2])}-${pad(m[1])}`;
  return /^\d{4}-\d{2}-\d{2}$/.test(s.trim()) ? s.trim() : null;
}

function parseTime(s) {
  const m = s.trim().match(/^(\d{1,2})(?:[:.\s](\d{2}))?$/);
  if (!m || Number(m[1]) > 23 || Number(m[2] || 0) > 59) return null;
  return `${pad(m[1])}:${m[2] || "00"}`;
}

function setPlace(place) {
  state.place = place;
  $("f-place").value = place ? placeLabel(place) : "";
  $("place-info").textContent = place ? `${place.lat.toFixed(4)}° s. š., ${place.lon.toFixed(4)}° v. d., pásmo ${place.tz}` : "";
}

function placeLabel(p) {
  return [p.name, p.region, p.country].filter(Boolean).join(", ");
}

function openForm(person) {
  state.editing = person;
  $("form-title").textContent = person ? "Upravit údaje" : "Nový člověk";
  $("show-only").hidden = !!person && !person.temp;
  $("delete-person").textContent = person?.temp ? "Zavřít bez uložení" : "Smazat";
  $("f-name").value = person?.name ?? "";
  $("f-sex").value = person?.sex ?? "";
  showNameHint();
  $("f-date").value = person ? czDate(person.date) : "";
  $("f-time").value = person?.time_known ? person.time : "";
  $("f-noknown").checked = person ? !person.time_known : false;
  $("f-time").disabled = $("f-noknown").checked;
  $("f-offset").value = person?.offset_min != null ? person.offset_min / 60 : "";
  $("f-note").value = person?.note ?? "";
  $("f-group").value = person?.group ?? (only() || "");
  $("form-error").textContent = "";
  $("delete-person").hidden = !person;
  setPlace(person?.place ?? null);
  hideFormSuggest();
  $("person-dialog").showModal();
  $("f-name").focus();
}

$("new-person").onclick = () => openForm(null);
$("edit-person").onclick = () => openForm(state.current);
$("cancel-person").onclick = () => $("person-dialog").close();
$("f-noknown").onchange = (e) => { $("f-time").disabled = e.target.checked; };

// ---------- printing ----------
let appTitle = document.title;

function sheetHtml() {
  const head = (title, line) => `<header><h1>${esc(title)}</h1><p>${esc(line)}</p>${state.current?.note ? `<p class="sheet-note">${esc(state.current.note)}</p>` : ""}</header>`;
  if (state.tab === "calendar") return state.calendar && head(state.calendar.title, state.calendar.label) + state.calendar.html;
  const p = state.panel;
  if (!p) return null;
  const events = subTabs()[state.sub] === "Kalendář" ? `<h3>Kalendář</h3>${$("panel").innerHTML}` : "";
  const aspects = p.aspects.length ? `<h3>Aspekty</h3>${aspectsTable(p)}` : "";
  return head(p.title, $("p-info").textContent) + `<div class="sheet-wheel">${p.svg}</div><h3>Polohy</h3>${planetsTable(p)}${aspects}${sectionsHtml(p)}${events}`;
}

$("print").onclick = () => {
  const html = sheetHtml();
  if (!html) return showError("Zatím není co tisknout.");
  $("print-sheet").innerHTML = html;
  // The title becomes the suggested file name when saving as PDF.
  document.title = (state.calendar && state.tab === "calendar" ? state.calendar.title : state.panel.title).replace(/[:/\\]/g, " -");
  window.print();
};
window.addEventListener("afterprint", () => {
  document.title = appTitle;
  $("print-sheet").innerHTML = "";
});

function readForm() {
  const err = (m) => { $("form-error").textContent = m; return null; };
  err("");
  const date = parseCzDate($("f-date").value);
  if (!date) return err("Datum zadejte například jako 14. 7. 1985.");
  const known = !$("f-noknown").checked;
  const time = known ? parseTime($("f-time").value) : "12:00";
  if (!time) return err("Čas zadejte například jako 14:30, nebo zaškrtněte, že ho neznáte.");
  if (!state.place) return err("Vyberte místo ze seznamu, který se nabídne při psaní.");
  const offsetText = $("f-offset").value.trim().replace(",", ".");
  const offset = offsetText === "" ? null : Math.round(Number(offsetText) * 60);
  if (Number.isNaN(offset)) return err("Posun zadejte jako číslo, například 1 nebo 5.5.");
  return { id: "", name: $("f-name").value.trim(), date, time, time_known: known, place: state.place, offset_min: offset, note: $("f-note").value, sex: $("f-sex").value, group: canonicalGroup($("f-group").value) };
}

$("person-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  const person = readForm();
  if (!person) return;
  const editing = state.editing;
  if (editing && !editing.temp) person.id = editing.id;
  try {
    const before = new Set(state.people.map((p) => p.id));
    const data = await invoke("save_person", { person });
    state.people = data.people;
    if (editing?.temp) setTemps(state.temps.filter((p) => p.id !== editing.id));
    $("person-dialog").close();
    // Saved into another group than the one shown: show all, so the person does not vanish from the list.
    const shown = $("group-filter").value;
    if (shown && (shown === NO_GROUP ? person.group : !sameGroup(person.group, shown))) $("group-filter").value = "";
    selectPerson(person.id || state.people.find((p) => !before.has(p.id))?.id);
  } catch (ex) {
    $("form-error").textContent = ex;
  }
});

// A person kept only in memory: nothing is written, so there is nothing to delete later.
$("show-only").onclick = async () => {
  const person = readForm();
  if (!person) return;
  person.name ||= "Neuložený člověk";
  try {
    await invoke("radix", { person, settings: state.settings });
  } catch (ex) {
    $("form-error").textContent = ex;
    return;
  }
  person.temp = true;
  const editing = state.editing;
  if (editing?.temp) {
    person.id = editing.id;
    setTemps(state.temps.map((p) => (p.id === editing.id ? person : p)));
  } else {
    person.id = `tmp-${Date.now()}`;
    setTemps([...state.temps, person]);
  }
  $("person-dialog").close();
  selectPerson(person.id);
};

$("save-temp").onclick = () => openForm(state.current);

$("delete-person").onclick = () => {
  if (state.editing.temp) {
    setTemps(state.temps.filter((p) => p.id !== state.editing.id));
    $("person-dialog").close();
    selectPerson("");
    return;
  }
  $("confirm-text").textContent = `Opravdu smazat: ${state.editing.name}? Tento krok nejde vrátit.`;
  $("confirm-dialog").showModal();
};
$("confirm-no").onclick = () => $("confirm-dialog").close();
$("confirm-yes").onclick = async () => {
  try {
    const data = await invoke("delete_person", { id: state.editing.id });
    state.people = data.people;
    $("confirm-dialog").close();
    $("person-dialog").close();
    state.current = null;
    selectPerson("");
  } catch (ex) {
    $("confirm-dialog").close();
    $("form-error").textContent = ex;
  }
};

// ---------- closing with unsaved people ----------
window.__TAURI__.event.listen("ask-close", () => {
  if ($("close-dialog").open) return;
  const names = state.temps.map((p) => p.name).join(", ");
  $("close-text").textContent = `${state.temps.length === 1 ? "Neuložený člověk" : "Neuložení lidé"} po zavření zmizí: ${names}.`;
  $("close-error").textContent = "";
  $("close-dialog").showModal();
});
$("close-back").onclick = () => $("close-dialog").close();
$("close-discard").onclick = () => invoke("close_app");
$("close-save").onclick = async () => {
  for (const { temp, ...person } of state.temps) {
    try {
      await invoke("save_person", { person: { ...person, id: "" } });
      setTemps(state.temps.filter((p) => p.id !== person.id));
    } catch (ex) {
      $("close-error").textContent = `${person.name} nejde uložit: ${ex}`;
      return;
    }
  }
  invoke("close_app");
};

// ---------- place autocomplete ----------
function placeSuggest(input, box, onPick, onEdit) {
  let items = [], active = -1, searchId = 0, debounce;
  const hide = () => {
    box.hidden = true;
    input.setAttribute("aria-expanded", "false");
    items = [];
    active = -1;
  };
  // Fixed position keeps the list clear of the scrolling form, and it flips up near the bottom edge.
  const position = () => {
    const r = input.getBoundingClientRect();
    const below = innerHeight - r.bottom - 8, above = r.top - 8;
    const up = below < 150 && above > below;
    Object.assign(box.style, {
      left: `${r.left}px`, width: `${r.width}px`, maxHeight: `${Math.min(250, up ? above : below)}px`,
      top: up ? "auto" : `${r.bottom + 2}px`, bottom: up ? `${innerHeight - r.top + 2}px` : "auto",
    });
  };
  document.addEventListener("scroll", () => { if (!box.hidden) position(); }, true);
  window.addEventListener("resize", () => { if (!box.hidden) position(); });
  const render = () => {
    box.innerHTML = items.map((s, i) => `<li role="option" data-i="${i}" class="${i === active ? "on" : ""}" aria-selected="${i === active}">${esc(s.name)} <small>${esc([s.region, s.country].filter(Boolean).join(", "))}</small></li>`).join("");
    box.hidden = !items.length;
    position();
    input.setAttribute("aria-expanded", String(!box.hidden));
    box.querySelector(".on")?.scrollIntoView({ block: "nearest" });
  };
  const pick = (i) => {
    const s = items[i];
    hide();
    onPick({ name: s.name, region: s.region, country: s.country, lat: s.lat, lon: s.lon, tz: s.tz });
  };
  input.addEventListener("input", () => {
    onEdit?.();
    clearTimeout(debounce);
    const q = input.value;
    debounce = setTimeout(async () => {
      const id = ++searchId;
      const res = await invoke("search_places", { query: q });
      if (id !== searchId) return;
      items = res;
      active = res.length ? 0 : -1;
      render();
    }, 80);
  });
  input.addEventListener("keydown", (e) => {
    if (box.hidden) return;
    if (e.key === "ArrowDown") { active = (active + 1) % items.length; render(); e.preventDefault(); }
    if (e.key === "ArrowUp") { active = (active - 1 + items.length) % items.length; render(); e.preventDefault(); }
    if (e.key === "Enter" && active >= 0) { pick(active); e.preventDefault(); }
    if (e.key === "Escape") { hide(); e.preventDefault(); e.stopPropagation(); }
  });
  input.addEventListener("blur", () => setTimeout(hide, 150));
  box.addEventListener("mousedown", (e) => {
    const li = e.target.closest("li");
    if (li) { pick(Number(li.dataset.i)); e.preventDefault(); }
  });
  return hide;
}

placeSuggest($("solar-place"), $("solar-suggest"), (place) => { solar.place = place; refresh(); });
$("solar-place").addEventListener("blur", () => setTimeout(() => { if (["solar", "lunar"].includes(state.tab) && document.activeElement !== $("solar-place")) renderSolarBar(); }, 200));

const hideFormSuggest = placeSuggest($("f-place"), $("suggest"), setPlace, () => {
  state.place = null;
  $("place-info").textContent = "";
});

// ---------- sky ----------
const sky = { place: null, date: isoDate(today), time: `${pad(today.getHours())}:${pad(today.getMinutes())}` };

function showSky() {
  state.current = null;
  state.tab = "sky";
  state.sub = 0;
  $("open-sky").classList.add("on");
  $("sky-bar").hidden = false;
  $("tabs").hidden = true;
  $("edit-person").hidden = true;
  $("save-temp").hidden = true;
  $("p-name").textContent = "Obloha";
  $("p-note").hidden = true;
  renderPeople();
  refresh();
}

function renderSkyBar() {
  $("sky-place").value = sky.place ? placeLabel(sky.place) : "";
  $("sky-date").value = sky.date;
  $("sky-time").value = sky.time;
  $("p-info").textContent = `${czDate(sky.date)}, ${sky.time}, ${sky.place?.name ?? ""}`;
}

function stepDay(days) {
  const [y, m, d] = sky.date.split("-").map(Number);
  sky.date = isoDate(new Date(y, m - 1, d + days, 12));
  refresh();
}

placeSuggest($("sky-place"), $("sky-suggest"), async (place) => {
  sky.place = place;
  refresh();
  await invoke("save_sky_place", { place }).catch(showError);
});
$("sky-place").addEventListener("blur", () => setTimeout(() => { if (document.activeElement !== $("sky-place")) renderSkyBar(); }, 200));
$("open-sky").onclick = showSky;
$("sky-prev").onclick = () => stepDay(-1);
$("sky-next").onclick = () => stepDay(1);
$("sky-now").onclick = () => {
  const now = new Date();
  sky.date = isoDate(now);
  sky.time = `${pad(now.getHours())}:${pad(now.getMinutes())}`;
  refresh();
};
$("sky-date").onchange = (e) => { if (e.target.value) { sky.date = e.target.value; refresh(); } };
$("sky-time").onchange = (e) => { if (e.target.value) { sky.time = e.target.value; refresh(); } };
document.addEventListener("keydown", (e) => {
  if (state.tab !== "sky" || document.querySelector("dialog[open]") || e.target.matches("input, select, textarea")) return;
  if (e.key === "ArrowLeft") { stepDay(-1); e.preventDefault(); }
  if (e.key === "ArrowRight") { stepDay(1); e.preventDefault(); }
});

// ---------- settings ----------
function applyTheme() {
  const dark = state.settings.dark;
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  $("theme").setAttribute("aria-checked", String(dark));
}

// Screen size zooms the whole window; the printed sheet takes its own size from --print-scale.
function applySizes(font = state.settings.font_scale, print = state.settings.print_scale) {
  document.documentElement.style.setProperty("--print-scale", String(print / 100));
  invoke("set_zoom", { scale: Number(font) }).catch(showError);
}

$("theme").onclick = async () => {
  state.settings = { ...state.settings, dark: !state.settings.dark };
  applyTheme();
  await invoke("save_settings", { settings: state.settings }).catch(showError);
};

$("open-settings").onclick = () => {
  $("s-house").value = state.settings.house_system;
  $("s-chiron").checked = state.settings.chiron;
  $("s-minor").checked = state.settings.minor_aspects;
  $("s-arc").checked = state.settings.solar_arc_angles;
  $("s-font").value = String(state.settings.font_scale);
  $("s-print").value = String(state.settings.print_scale);
  $("backup-result").textContent = "";
  showBackupInfo();
  $("settings-dialog").showModal();
};

async function showBackupInfo() {
  const [count, newest] = await invoke("backup_info");
  $("backup-info").textContent = count
    ? `Program si při první změně v každém dni sám uloží kopii. Teď jich má ${count}, poslední je z ${czDate(newest)}. Najdete je při načítání zálohy.`
    : "Program si při první změně v každém dni sám uloží kopii. Zatím žádnou nemá.";
}

$("backup-save").onclick = async () => {
  try {
    const count = await invoke("export_backup");
    if (count !== null) $("backup-result").textContent = `Záloha je uložená. Počet lidí v ní: ${count}.`;
  } catch (e) {
    $("backup-result").textContent = e;
  }
};

$("backup-load").onclick = async () => {
  try {
    const result = await invoke("import_backup");
    if (!result) return;
    const [data, added] = result;
    state.people = data.people;
    renderPeople();
    $("backup-result").textContent = added ? `Hotovo. Přidaní lidé: ${added}. Nikdo jiný se nezměnil.` : "Všichni lidé ze zálohy už v programu jsou.";
    showBackupInfo();
  } catch (e) {
    $("backup-result").textContent = e;
  }
};
// The screen size is tried out at once; closing without Hotovo goes back to the saved size.
$("s-font").onchange = () => applySizes(Number($("s-font").value));
$("settings-dialog").addEventListener("close", () => applySizes());
$("settings-save").onclick = async () => {
  const settings = {
    ...state.settings, house_system: $("s-house").value, chiron: $("s-chiron").checked, minor_aspects: $("s-minor").checked,
    solar_arc_angles: $("s-arc").checked, font_scale: Number($("s-font").value), print_scale: Number($("s-print").value),
  };
  const data = await invoke("save_settings", { settings });
  state.settings = data.settings;
  $("settings-dialog").close();
  if (state.current) refresh();
};

// ---------- updates ----------
// One question to the release page at start. Without internet nothing is shown.
async function checkUpdate() {
  const update = await invoke("check_update").catch(() => null);
  if (!update) return;
  $("update-text").textContent = `Je k dispozici nová verze ${update.version.replace(/\.0$/, "")}. Lidé a nastavení zůstanou.`;
  $("update-bar").hidden = false;
}
$("update-later").onclick = () => ($("update-bar").hidden = true);
$("update-now").onclick = async () => {
  $("update-now").disabled = $("update-later").disabled = true;
  $("update-text").textContent = "Stahuji novou verzi. Program se pak sám zavře, nainstaluje a znovu spustí.";
  try {
    await invoke("install_update");
  } catch (e) {
    $("update-text").textContent = String(e);
    $("update-now").disabled = $("update-later").disabled = false;
  }
};

// ---------- start ----------
(async () => {
  const [data, systems, version] = await Promise.all([invoke("load_data"), invoke("house_systems"), window.__TAURI__.app.getVersion()]);
  $("app-version").textContent = `Verze ${version.replace(/\.0$/, "")}`;
  state.people = data.people;
  state.settings = data.settings;
  applyTheme();
  applySizes();
  $("s-house").innerHTML = systems.map(([k, n]) => `<option value="${k}">${esc(n)}</option>`).join("");
  sky.place = data.sky_place ?? (await invoke("search_places", { query: "Praha" }))[0];
  showSky();
  checkUpdate();
})();
