// Help centre content. Topics are plain data; figures come from HELP_FIGURES in help.js.
// `{{name}}` in a text inserts a figure, `<a data-topic="id">` opens another topic.
// Everything here describes what the program really does, so a change in the calculations must be
// written here too. Numbers in the help text (orbs, limits) mirror core/src/data.rs and views.rs.

const HELP_AREAS = [
  ["start", "Začínáme"],
  ["work", "Práce s programem"],
  ["views", "Pohledy"],
  ["calc", "Jak se počítá"],
  ["data", "Nastavení a data"],
];

const SKIP_WITHOUT_PERSON = "Tuto část prohlídky uvidíte, až vyberete člověka.";

// A tour step points at a `data-help` element. `tab` and `sub` switch the view first. A step whose
// element is not on screen is skipped.
const HELP_TOURS = {
  start: [
    { at: "side-brand", title: "Program a režim", text: "Přepínačem vpravo změníte denní a noční vzhled." },
    { at: "sky", title: "Obloha", text: "Ukáže, jak obloha vypadá právě teď, nebo v kterýkoli den a na kterémkoli místě. Program se tímto pohledem otevírá." },
    { at: "new-person", title: "Nový člověk", text: "Sem zadáte jméno, datum, čas a místo narození." },
    { at: "filter", title: "Hledání", text: "Hledá ve jménech, místech, poznámkách i skupinách. Háčky a čárky psát nemusíte." },
    { at: "groups", title: "Skupiny", text: "Když lidem dáte skupinu, třeba Rodina nebo Klienti, objeví se tady výběr. Ukáže jen jednu skupinu, nebo všechny pod nadpisy." },
    { at: "people", title: "Seznam lidí", text: "Kliknutím na jméno otevřete horoskop daného člověka." },
    { at: "tabs", title: "Pohledy na člověka", text: "Horoskop, tranzity, progrese, solární návrat, kalendář a vztah. Každý má vlastní téma v nápovědě.", tab: "radix" },
    { at: "wheel", title: "Kolo", text: "Obrázek horoskopu. Co znamenají barvy a čáry, popisuje téma Jak číst kolo." },
    { at: "detail", title: "Planety, Aspekty, Výklad", text: "Vpravo jsou k tomu tabulky a texty. Přepínají se tlačítky nahoře." },
    { at: "print", title: "Tisk a PDF", text: "Vytiskne právě otevřený pohled, nebo ho uloží jako PDF." },
    { at: "settings", title: "Nastavení", text: "Systém domů, Chiron, vedlejší aspekty, progrese os a záloha lidí." },
    { at: "help", title: "Nápověda", text: "Otazník nahoře otevře nápovědu, stejně jako klávesa F1. Ukáže téma k tomu, co máte právě na obrazovce." },
  ],
  sky: [
    { at: "sky", title: "Obloha", text: "Tímto tlačítkem se k obloze vrátíte z horoskopu kteréhokoli člověka." },
    { at: "sky-place", title: "Místo", text: "Začněte psát a vyberte město ze seznamu. Program si ho zapamatuje." },
    { at: "sky-steps", title: "Den a čas", text: "Tlačítka posouvají o den. Totéž umí šipky vlevo a vpravo na klávesnici. Teď vrátí aktuální okamžik." },
    { at: "wheel", title: "Kolo", text: "Ukazuje polohy pro zvolený okamžik a místo." },
  ],
  radix: [
    { at: "tabs", title: "Horoskop", text: "První záložka je radix, tedy obloha v okamžiku narození.", tab: "radix" },
    { at: "wheel", title: "Kolo", text: "Vlevo je Ascendent, nahoře zhruba MC." },
    { at: "detail", title: "Planety", text: "Tabulka poloh, domů a postavení ve znamení.", sub: 0 },
    { at: "detail", title: "Aspekty", text: "Úhly mezi tělesy a jejich orby. Nejsilnější jsou nahoře.", sub: 1 },
    { at: "detail", title: "Výklad", text: "Česká interpretace všeho, co v horoskopu je.", sub: 2 },
  ],
  transit: [
    { at: "tabs", title: "Tranzity", text: "Co planety dělají teď vzhledem k horoskopu z narození.", tab: "transit" },
    { at: "controls", title: "Den a čas", text: "Zvolte okamžik, pro který se má počítat. Teď vrátí aktuální čas." },
    { at: "wheel", title: "Dvě kola", text: "Uvnitř je radix, vně planety pro zvolený den." },
    { at: "detail", title: "Co teď působí", text: "Tranzity s orbem do 2°. Nejvýraznější jsou nahoře.", sub: 2 },
  ],
  prog: [
    { at: "tabs", title: "Progrese", text: "Pomalý vnitřní vývoj člověka: jeden den po narození odpovídá jednomu roku života.", tab: "prog" },
    { at: "controls", title: "Den", text: "Zvolte den, ke kterému se má progrese spočítat." },
    { at: "detail", title: "Výklad", text: "Progrese s orbem do 1°.", sub: 2 },
    { at: "detail", title: "Kalendář", text: "Přesné zásahy po měsících nebo za celý rok.", sub: 3 },
  ],
  solar: [
    { at: "tabs", title: "Solární návrat", text: "Horoskop okamžiku, kdy se Slunce vrátí na svou polohu z narození.", tab: "solar" },
    { at: "solar-bar", title: "Rok a místo", text: "Zvolte rok. Pokud člověk v den návratu pobývá jinde, zadejte to místo. Tlačítko Místo narození ho vrátí." },
    { at: "wheel", title: "Kolo", text: "Solární horoskop platí zhruba do dalšího návratu." },
  ],
  calendar: [
    { at: "tabs", title: "Kalendář", text: "Přehled přesných tranzitů za měsíc nebo celý rok.", tab: "calendar" },
    { at: "controls", title: "Období", text: "Zvolte měsíc nebo Celý rok. Jen důležité skryje rychlé planety, zbude Jupiter a pomalejší." },
    { at: "calendar", title: "Seznam", text: "Každý řádek je den a čas, kdy je aspekt přesný. Barevná tečka ukazuje druh aspektu." },
  ],
  vztah: [
    { at: "tabs", title: "Vztah s…", text: "Porovnání dvou lidí.", tab: "rel" },
    { at: "controls", title: "Druhý člověk a zobrazení", text: "Vyberte druhého člověka. Zobrazení přepíná mezi dvěma koly přes sebe a složeným horoskopem." },
  ],
};

const HELP_TOPICS = [
  {
    id: "start", area: "start", title: "Jak se v programu vyznat", tour: "start",
    summary: "Co je kde na obrazovce a kde začít.",
    keys: "úvod přehled začátek orientace obrazovka",
    html: `
<p>Kosmogram počítá a vykládá horoskopy. Vlevo je seznam lidí, vpravo pracovní plocha pro toho, koho jste vybrali.</p>
{{screen}}
<ol class="legend">
  <li><b>Obloha</b> ukáže polohy pro kterýkoli okamžik, bez konkrétního člověka.</li>
  <li><b>Nový člověk a seznam lidí.</b> Kliknutím na jméno otevřete horoskop.</li>
  <li><b>Nastavení</b> je dole v levém sloupci. Nápověda je otazník nahoře vedle názvu programu.</li>
  <li><b>Záložky</b> přepínají pohledy na člověka, od horoskopu po vztah.</li>
  <li><b>Kolo</b> je obrázek horoskopu.</li>
  <li><b>Tabulky a výklad</b> jsou vedle kola, přepínají se tlačítky Planety, Aspekty a Výklad.</li>
</ol>
<h3>První kroky</h3>
<ol>
  <li>Klikněte na <b>+ Nový člověk</b> a vyplňte údaje. Podrobnosti jsou v tématu <a data-topic="lide">Lidé</a>.</li>
  <li>Program ukáže horoskop. Začněte tématem <a data-topic="kolo">Jak číst kolo</a>.</li>
  <li>Otevřete <b>Výklad</b> a procházejte záložky.</li>
</ol>
<p class="tip">Otazník nahoře nebo klávesa <kbd>F1</kbd> otevře nápovědu k tomu, co máte právě na obrazovce. Otazník u nastavení a ve formuláři vysvětlí jednu konkrétní volbu.</p>`,
  },
  {
    id: "novinky", area: "start", title: "Co je nového",
    summary: "Co přinesla která verze.",
    keys: "novinky změny verze 1.0 1.0.1 1.0.2 co je nového",
    html: `
<h3>Verze 1.0.2</h3>
<ul>
  <li><b>Skupiny lidí.</b> Ve formuláři dáte člověku skupinu, třeba Rodina, Klienti nebo Známí. Nad seznamem pak vyberete, kterou skupinu chcete vidět. <a data-topic="lide">Více</a>.</li>
</ul>
<h3>Verze 1.0.1</h3>
<ul>
  <li><b>Velikost písma.</b> V Nastavení zvolíte větší písmo na obrazovce a zvlášť velikost textu při tisku a v PDF. <a data-topic="nastaveni">Více</a>.</li>
</ul>
<h3>Verze 1.0</h3>
<ul>
  <li><b>Horoskop, tranzity, progrese, solární návrat, kalendář a vztahy.</b> Polohy se počítají z dat NASA JPL přímo v programu, bez internetu.</li>
  <li><b>České výklady.</b> Všechny texty jsou napsané česky, celkem 5008 kombinací těles, znamení, domů a aspektů.</li>
  <li><b>Jména ve vztazích.</b> Výklady vztahů používají jména ze zadání a správně je skloňují. Rod pozná program podle jména, ve formuláři ho můžete změnit.</li>
  <li><b>Fáze Měsíce.</b> Výklad píše astrologickou fázi a vedle ní, jak Měsíc vypadal doopravdy. <a data-topic="radix">Více</a>.</li>
  <li><b>Nápověda.</b> Otazník nahoře otevře tuto nápovědu s hledáním, obrázky, prohlídkou programu a porovnáním systémů domů.</li>
  <li><b>Lidé zůstávají.</b> Lidé a nastavení ze starších verzí se načtou beze změny.</li>
</ul>`,
  },
  {
    id: "lide", area: "work", title: "Lidé: přidání, úprava, hledání",
    summary: "Formulář, skupiny, rod ve výkladech, Jen zobrazit, mazání.",
    keys: "nový člověk formulář jméno datum čas místo poznámka uložit smazat rod muž žena skloňování neuložený dočasný jen zobrazit skupina skupiny rodina klienti známí filtr",
    html: `
<h3>Nový člověk</h3>
<dl>
  <dt>Jméno</dt><dd>Pod polem hned uvidíte, jak bude jméno znít ve výkladech, například „Slunce Petra“. Pokud je tvar špatně, opravte <b>Rod ve výkladech</b>.</dd>
  <dt>Datum narození</dt><dd>Například 14. 7. 1985. Program počítá roky 1800 až 2399.</dd>
  <dt>Čas narození</dt><dd>Například 14:30. Pokud ho neznáte, zaškrtněte <b>Čas narození neznám</b>. Co to změní, popisuje téma <a data-topic="cas">Čas narození a časová pásma</a>.</dd>
  <dt>Místo narození</dt><dd>Začněte psát a vyberte město ze seznamu. Výběr určí souřadnice i časové pásmo.</dd>
  <dt>Skupina</dt><dd>Nepovinná, například Rodina, Klienti nebo Známí. Napište název, nebo vyberte skupinu, kterou už máte. Velká a malá písmena nevadí, „rodina“ se uloží do skupiny Rodina.</dd>
  <dt>Poznámka</dt><dd>Cokoli pro vás. Ukáže se pod jménem, vytiskne se a dá se v ní hledat.</dd>
</dl>
<h3>Skupiny v seznamu</h3>
<p>Jakmile má aspoň jeden člověk skupinu, objeví se nad seznamem výběr. <b>Všechny skupiny</b> ukáže všechny lidi pod nadpisy skupin, lidé bez skupiny jsou na konci. Když vyberete jednu skupinu, uvidíte jen ji a nový člověk do ní rovnou patří. Hledání funguje i v názvech skupin.</p>
<p>Skupina zmizí sama, když v ní nikdo nezůstane. Přejmenujete ji tak, že lidem ve formuláři přepíšete skupinu. Záloha skupiny ukládá a obnova je vrátí, člověku, který už skupinu má, ji nepřepíše.</p>
<h3>Uložit, nebo Jen zobrazit</h3>
<p><b>Uložit</b> člověka přidá do seznamu natrvalo. <b>Jen zobrazit</b> spočítá horoskop bez uložení, člověk se objeví nahoře pod nadpisem Neuloženo. Při zavírání se program zeptá, jestli ho chcete uložit.</p>
<h3>Úprava a mazání</h3>
<p>Tlačítko <b>Upravit údaje</b> je nad kolem. Ve stejném formuláři je i <b>Smazat</b>, program se ještě zeptá. Smazání nejde vrátit, pomůže jen <a data-topic="zaloha">záloha</a>.</p>`,
  },
  {
    id: "kolo", area: "work", title: "Jak číst kolo",
    summary: "Znamení, domy, osy, aspekty a barvy na obrázku.",
    keys: "kolo obrázek horoskop legenda barvy čáry symboly živly znamení domy ascendent mc",
    html: `
<p>Kolo je pohled na oblohu ze Země. Znamení běží proti směru hodinových ručiček a Ascendent je vždy vlevo.</p>
{{wheel}}
<ol class="legend">
  <li><b>Vnější pás: znamení.</b> Barva ukazuje živel, viz níže.</li>
  <li><b>Tělesa.</b> Symbol, stupeň a případně značka zpětného pohybu.</li>
  <li><b>Domy.</b> Dvanáct výsečí s čísly. Rozdělení závisí na <a data-topic="domy">systému domů</a>.</li>
  <li><b>Ascendent</b> je bod, který v okamžiku narození vycházel na východě.</li>
  <li><b>MC, Medium coeli,</b> je nejvyšší bod dráhy planet na obloze.</li>
  <li><b>Čáry uprostřed: aspekty.</b> Čím silnější čára, tím je aspekt přesnější.</li>
</ol>
<h3>Barvy znamení</h3>
{{elements}}
<h3>Barvy aspektů</h3>
{{aspectColors}}
<h3>Symboly</h3>
{{glyphs}}
<h3>Dvě kola přes sebe</h3>
{{double}}
<p>U tranzitů, progresí a synastrie jsou vnitřní kolo a vnější prstenec dvě různé sady poloh. Čáry pak spojují tělesa z vnějšího prstence s tělesy uvnitř.</p>
<p>Bez času narození nejsou známé domy ani osy, proto je vlevo začátek Berana.</p>`,
  },
  {
    id: "tabulky", area: "work", title: "Planety, Aspekty, Výklad",
    summary: "Co znamenají sloupce a slova v tabulkách.",
    keys: "tabulka planety aspekty výklad orb sbíhá rozbíhá těsný volný vládce vyhnanství povýšení pád zpětný retrográdní R",
    html: `
<h3>Planety</h3>
<dl>
  <dt>Poloha</dt><dd>Stupně a minuty ve znamení, například 12°34′ Lev.</dd>
  <dt>Dům</dt><dd>Oblast života, kde těleso působí. Hroty domů jsou v tabulce pod planetami.</dd>
  <dt>R</dt><dd>Zpětný pohyb. Těleso se při pohledu ze Země zdánlivě vrací.</dd>
  <dt>Vládce</dt><dd>Těleso je ve znamení, kterému vládne, a působí přirozeně.</dd>
  <dt>Vyhnanství</dt><dd>Těleso je ve znamení naproti svému, působí méně pohodlně.</dd>
  <dt>Povýšení</dt><dd>Znamení, kde se těleso silně projevuje.</dd>
  <dt>Pád</dt><dd>Znamení naproti povýšení, projev je obtížnější.</dd>
</dl>
<h3>Aspekty</h3>
<dl>
  <dt>Orb</dt><dd>O kolik stupňů se úhel liší od přesného aspektu. Menší orb znamená výraznější aspekt.</dd>
  <dt>Sbíhá se, rozbíhá se</dt><dd>Sbíhající aspekt se teprve přesní, rozbíhající už přesností prošel.</dd>
  <dt>Těsný, volný</dt><dd>Ve výkladu. Těsný je v první třetině povoleného orbu, volný v poslední.</dd>
</dl>
<p>Tabulka je seřazená podle síly, nejvýraznější aspekty jsou nahoře. Podrobnosti jsou v tématu <a data-topic="aspekty">Aspekty a orby</a>.</p>
<h3>Výklad</h3>
<p>Texty popisují sklony a témata, ne jisté události. Podrobnosti jsou v tématu <a data-topic="vyklady">Odkud jsou výklady</a>.</p>`,
  },
  {
    id: "tisk", area: "work", title: "Tisk a uložení do PDF",
    summary: "Co se vytiskne a jak vznikne PDF.",
    keys: "tisk tiskárna pdf vytisknout uložit papír velikost písma větší",
    html: `
<p>Tlačítko <b>Vytisknout nebo uložit PDF</b> vytiskne to, co máte právě otevřené. Na listu je jméno, poznámka, kolo, tabulka poloh, aspekty a výklad. V Kalendáři se tiskne seznam událostí.</p>
<p>Otevře se okno tisku Windows. Pro PDF v něm místo tiskárny zvolte <b>Uložit jako PDF</b> nebo <b>Microsoft Print to PDF</b>. Název souboru program předvyplní podle pohledu a jména.</p>
<p>Tisk je vždy světlý, i když máte zapnutý noční režim.</p>
<p>Velikost textu na listu nastavíte v Nastavení, volba <b>Při tisku a v PDF</b>. Velikost písma na obrazovce tisk nemění.</p>`,
  },

  {
    id: "obloha", area: "views", title: "Obloha", tour: "sky",
    summary: "Polohy pro libovolný den a místo, bez člověka.",
    keys: "dnes teď aktuální nebe den zpět dopředu",
    html: `
<p>Obloha ukazuje polohy těles pro zvolený okamžik a místo. Hodí se pro pohled na to, co se děje dnes, nebo pro rychlou kontrolu data.</p>
<ul>
  <li><b>Místo</b> určuje domy, Ascendent a MC a také pásmo, ve kterém se čte zadaný čas. Jiné pásmo znamená jiný okamžik, takže se může posunout hlavně Měsíc.</li>
  <li><b>Den zpět, Den dopředu</b> posouvají o den, totéž umí šipky na klávesnici.</li>
  <li><b>Teď</b> nastaví aktuální den a čas.</li>
</ul>
<p>Čas se zadává podle pásma zvoleného místa. Obloha nemá výklad, jen tabulky Planety a Aspekty.</p>`,
  },
  {
    id: "radix", area: "views", title: "Horoskop (radix)", tour: "radix",
    summary: "Základní horoskop z okamžiku narození.",
    keys: "radix natální narození základ fáze měsíce úplněk novoluní čtvrť srpek",
    html: `
<p>Radix je obloha v okamžiku a na místě narození. Je základem všech ostatních pohledů.</p>
<p>Výklad má čtyři části: <b>Celkový obraz</b>, <b>Planety ve znameních a domech</b>, <b>Domy a jejich vládci</b> a <b>Aspekty</b>.</p>
<h3>Fáze Měsíce</h3>
<p>Celkový obraz začíná fází Měsíce. Astrologie dělí měsíční cyklus na osm fází po 45°. Každá začíná přesným okamžikem a trvá dál, takže <b>fáze úplňku</b> sahá od úplňku až zhruba do 3,7 dne po něm. Proto se může lišit od kalendáře, který úplněk píše jen v jeden den.</p>
<p>V závorce je, jak Měsíc vypadal doopravdy, například „Měsíc 3 dny po úplňku, osvětlený z 92 %, ubývá, 213° od Slunce“. Výklad se řídí astrologickou fází.</p>
<p>Pod tabulkou poloh jsou údaje, se kterými program počítal: čas, posun od UTC a souřadnice. Pokud je poloha nebo čas nejistý, přečtete si tam upozornění, viz <a data-topic="cas">Čas narození a časová pásma</a>.</p>`,
  },
  {
    id: "tranzity", area: "views", title: "Tranzity", tour: "transit",
    summary: "Co planety na obloze dělají s horoskopem z narození.",
    keys: "tranzit přechod aktuální působí teď orb 2",
    html: `
<p>Tranzit je aspekt mezi planetou na dnešní obloze a tělesem nebo osou v radixu. Ukazuje, která témata jsou zrovna ve hře.</p>
{{double}}
<ul>
  <li>Uvnitř je radix, vně polohy pro zvolený den a čas.</li>
  <li>Do výkladu <b>Co teď působí</b> se dostanou tranzity s <b>orbem do 2°</b>.</li>
  <li><b>Měsíc</b> se do výkladu nepočítá, pohybuje se tak rychle, že tranzit trvá jen hodiny.</li>
  <li>Nahoře jsou nejvýraznější tranzity: pomalé planety a přesnější aspekty mají větší váhu.</li>
</ul>
<p>Den a čas se zadává v českém čase. Pro přehled za delší období použijte <a data-topic="kalendar">Kalendář</a>.</p>`,
  },
  {
    id: "progrese", area: "views", title: "Progrese", tour: "prog",
    summary: "Jeden den po narození odpovídá jednomu roku života.",
    keys: "sekundární progrese vývoj den za rok orb 1",
    html: `
<p>Sekundární progrese popisují pomalý vnitřní vývoj. Vychází z pravidla, že <b>jeden den po narození odpovídá jednomu roku života</b>.</p>
{{dayYear}}
<ul>
  <li>Uvnitř je radix, vně progresivní polohy.</li>
  <li>Výklad ukazuje progrese s <b>orbem do 1°</b>.</li>
  <li><b>Kalendář</b> vypíše přesné zásahy pro zvolený měsíc nebo rok. Sem patří i progresivní Měsíc a, pokud je znám čas narození, osy.</li>
</ul>
<p>Pohyb Ascendentu a MC se dá počítat dvěma způsoby. Rozdíl popisuje téma <a data-topic="osy">Progrese os</a>.</p>`,
  },
  {
    id: "solar", area: "views", title: "Solární návrat", tour: "solar",
    summary: "Horoskop okamžiku, kdy se Slunce vrátí na svou polohu.",
    keys: "solar narozeniny rok návrat revoluce místo",
    html: `
<p>Jednou za rok, kolem narozenin, se Slunce vrátí přesně na stupeň, kde bylo při narození. Horoskop tohoto okamžiku ukazuje témata na zhruba další rok.</p>
<ul>
  <li><b>Rok</b> určuje, který návrat se počítá.</li>
  <li><b>Místo v den návratu</b> mění domy a osy, polohy planet zůstávají stejné. Zadejte místo, kde člověk v ten den skutečně pobývá.</li>
  <li><b>Místo narození</b> vrátí výchozí místo.</li>
</ul>
<p>Přesný den a čas návratu je v tabulce Planety pod polohami. Může se o den lišit od data narození, protože rok není přesně 365 dní.</p>
<p>Bez času narození nejde určit přesný okamžik, proto chybí domy, Ascendent a MC.</p>`,
  },
  {
    id: "kalendar", area: "views", title: "Kalendář", tour: "calendar",
    summary: "Přesné tranzity za měsíc nebo rok.",
    keys: "kalendář přehled rok měsíc důležité událost přesný zpětně",
    html: `
<p>Kalendář hledá okamžiky, kdy je tranzit přesný. Každý řádek má den, čas, co se děje a krátký výklad.</p>
<ul>
  <li><b>Jen důležité</b> nechá Jupitera a pomalejší tělesa. Rychlejší planety prochází aspekty často a přehled by zahltily.</li>
  <li><b>(zpětně)</b> znamená, že planeta je v tu chvíli ve zpětném pohybu. Pomalá planeta se tak může ke stejnému bodu vrátit vícekrát.</li>
  <li>Časy jsou v českém čase. Tranzitní Měsíc se nepočítá.</li>
</ul>
<p>Přesný okamžik je jen vrchol. Tranzit působí už před ním i po něm, u pomalých planet týdny až měsíce.</p>`,
  },
  {
    id: "vztah", area: "views", title: "Vztah: synastrie a složený horoskop", tour: "vztah",
    summary: "Dvě kola přes sebe, nebo jeden společný horoskop.",
    keys: "vztah partner synastrie složený kompozit composite dva lidé jména",
    html: `
<p>Záložka <b>Vztah s…</b> porovnává vybraného člověka s druhým. Druhého vyberete v poli nad kolem.</p>
<h3>Dvě kola přes sebe (synastrie)</h3>
<p>Uvnitř je horoskop prvního člověka, vně fialově druhého. Ukazuje, jak na sebe dva lidé působí: aspekty mezi jejich tělesy a planety jednoho v domech druhého.</p>
<p>Tabulka Planety ukazuje planety druhého člověka v domech prvního. Výklad popisuje oba směry, pokud jsou známé oba časy narození.</p>
<h3>Složený horoskop</h3>
{{composite}}
<p>Složený horoskop tvoří středy mezi polohami obou lidí a popisuje vztah samotný. Každé těleso i hrot domu je středem na kratším oblouku mezi oběma polohami.</p>
<p>Tělesa tu nikomu nepatří, proto výklad nikoho jmenovitě neurčuje a role jsou zaměnitelné. Domy jsou k dispozici, jen když jsou známé oba časy narození.</p>`,
  },

  {
    id: "domy", area: "calc", title: "Systémy domů", compare: true,
    summary: "Osm způsobů, jak rozdělit horoskop, a porovnání na člověku.",
    keys: "placidus koch rovné celá znamení regiomontanus campanus porphyrius alcabitius hroty polární",
    html: `
<p>Domy dělí oblohu na dvanáct oblastí života. Rozdělit kouli na dvanáct dílů jde více způsoby a žádný není jediný správný.</p>
{{houseKinds}}
<p><b>Co se změnou systému změní:</b> hroty domů a to, ve kterém domě planeta leží, a tím i text o planetě v domě. <b>Co se nezmění:</b> polohy ve znameních, Ascendent, MC a aspekty.</p>
<table class="compare-sys">
  <tr><th>Systém</th><th>Jak dělí</th><th>Poznámka</th></tr>
  <tr><td><b>Placidus</b></td><td>Podle času: dobu, za kterou bod na obloze vystoupá od obzoru k MC, dělí na třetiny.</td><td>Výchozí a dnes nejrozšířenější.</td></tr>
  <tr><td><b>Koch</b></td><td>Také podle času, ale vychází z doby vystoupání samotného MC.</td><td>Oblíbený zejména v Německu.</td></tr>
  <tr><td><b>Rovné domy</b></td><td>Od Ascendentu po 30°.</td><td>MC nemusí být na hrotu 10. domu.</td></tr>
  <tr><td><b>Celá znamení</b></td><td>Každý dům je jedno celé znamení, 1. dům je znamení Ascendentu.</td><td>Nejstarší způsob. Ascendent i MC mohou být uvnitř domu.</td></tr>
  <tr><td><b>Regiomontanus</b></td><td>Dělí nebeský rovník po 30°.</td><td>Tradiční pro hodinovou astrologii.</td></tr>
  <tr><td><b>Campanus</b></td><td>Dělí po 30° kružnici procházející východem, nadhlavníkem a západem.</td><td>Vychází z místa pozorovatele.</td></tr>
  <tr><td><b>Porphyrius</b></td><td>Každou ze čtyř čtvrtin mezi Ascendentem a MC dělí na tři stejné díly.</td><td>Velmi starý a jednoduchý.</td></tr>
  <tr><td><b>Alcabitius</b></td><td>Dělí na třetiny denní a noční dráhu stupně Ascendentu.</td><td>Středověký systém.</td></tr>
</table>
<p class="tip">Blízko polárních kruhů a za nimi Placidus a Koch nejdou spočítat. Program to oznámí, zvolte pak například Rovné domy nebo Celá znamení.</p>
<p>Systém domů se mění v <a data-topic="nastaveni">Nastavení</a> a platí pro všechny lidi.</p>`,
  },
  {
    id: "aspekty", area: "calc", title: "Aspekty a orby",
    summary: "Které aspekty program počítá a s jakou tolerancí.",
    keys: "aspekt orb konjunkce opozice kvadratura trigon sextil kvinkunx polosextil polokvadrát seskvikvadrát vedlejší hlavní tolerance",
    html: `
<p>Aspekt je význačný úhel mezi dvěma tělesy. Orb je tolerance, o kolik se úhel smí lišit, aby aspekt ještě platil.</p>
{{aspectAngles}}
<h3>Hlavní aspekty</h3>
<table>
  <tr><th>Aspekt</th><th>Úhel</th><th>Orb</th></tr>
  <tr><td>Konjunkce</td><td>0°</td><td>7°</td></tr>
  <tr><td>Opozice</td><td>180°</td><td>7°</td></tr>
  <tr><td>Kvadratura</td><td>90°</td><td>7°</td></tr>
  <tr><td>Trigon</td><td>120°</td><td>7°</td></tr>
  <tr><td>Sextil</td><td>60°</td><td>6°</td></tr>
</table>
<h3>Vedlejší aspekty</h3>
<p>Zapínají se v <a data-topic="nastaveni">Nastavení</a>.</p>
<table>
  <tr><th>Aspekt</th><th>Úhel</th><th>Orb</th></tr>
  <tr><td>Kvinkunx</td><td>150°</td><td>3°</td></tr>
  <tr><td>Polosextil</td><td>30°</td><td>3°</td></tr>
  <tr><td>Polokvadrát</td><td>45°</td><td>3°</td></tr>
  <tr><td>Seskvikvadrát</td><td>135°</td><td>3°</td></tr>
</table>
<h3>Úpravy orbu</h3>
<ul>
  <li><b>Slunce a Měsíc</b> mají orb o 1° větší, u aspektu dvou takových těles se to sečte.</li>
  <li><b>Severní uzel</b> má orb nejvýš 2°, s tímto příplatkem o něco víc.</li>
  <li><b>Tranzity</b> se ve výkladu ukazují jen do 2°, <b>progrese</b> jen do 1°.</li>
</ul>
<p>Hodnoty odpovídají původnímu programu Astrolog. V programu se nedají měnit.</p>`,
  },
  {
    id: "osy", area: "calc", title: "Progrese os: Astrolog, nebo solární oblouk",
    summary: "Dva způsoby pohybu Ascendentu, MC a domů v progresi.",
    keys: "solární oblouk astrolog osy progresivní ascendent mc rychlé domy nastavení",
    html: `
<p>Tělesa se v progresi počítají vždy stejně. Liší se jen Ascendent, MC a hroty domů.</p>
{{axes}}
<table>
  <tr><th></th><th>Jako v Astrologu (výchozí)</th><th>Solárním obloukem</th></tr>
  <tr><td><b>Jak se počítá</b></td><td>Celý horoskop pro progresivní okamžik, tedy se skutečným otočením Země.</td><td>MC se posouvá o tolik, o kolik postoupilo progresivní Slunce, ostatní hroty se dopočítají.</td></tr>
  <tr><td><b>Rychlost</b></td><td>Za rok oběhnou celý kruh.</td><td>Asi 1° za rok.</td></tr>
  <tr><td><b>Zásah osy</b></td><td>Trvá jen několik dní.</td><td>Trvá kolem dvou let, orb je 1° na každou stranu.</td></tr>
  <tr><td><b>Výklad</b></td><td>U os vlastní texty pro krátké období.</td><td>U os texty pro delší období.</td></tr>
</table>
<p>Pro dlouhá životní období se u os běžně používá solární oblouk. Výchozí zůstává způsob původního programu, aby výsledky seděly se starým Astrologem.</p>
<p>Přepíná se v <a data-topic="nastaveni">Nastavení</a> volbou <b>Progresivní osy a domy solárním obloukem</b>.</p>`,
  },
  {
    id: "cas", area: "calc", title: "Čas narození a časová pásma",
    summary: "Neznámý čas, letní čas, upozornění a ruční posun.",
    keys: "čas neznámý poledne letní zimní pásmo utc posun ruční historie válka slovensko upozornění dvakrát neexistoval místní",
    html: `
<h3>Když čas narození neznáte</h3>
<p>Program počítá pro poledne. Chybí domy, Ascendent a MC a Měsíc může být až o 7° jinde, protože se pohybuje rychle. Ostatní tělesa se posunou jen málo. Přesný čas je nejcennější údaj, i 10 minut může posunout Ascendent o několik stupňů.</p>
<h3>Jak program pozná čas</h3>
<p>Čas se zadává tak, jak ho ukazovaly hodiny. Podle místa a data program dohledá časové pásmo i letní čas a převede čas na světový. Pro Česko a Slovensko jsou historické změny času opravené podle zákonů.</p>
<p>Použitý posun je v tabulce Planety u údajů o narození, například +1 nebo +2.</p>
<h3>Upozornění pod tabulkou</h3>
<dl>
  <dt>Čas nastal dvakrát</dt><dd>Při návratu z letního času se hodina opakuje. Program použije první z nich.</dd>
  <dt>Čas neexistoval</dt><dd>Při přechodu na letní čas se hodina přeskočí. Program použije čas o hodinu pozdější.</dd>
  <dt>Pásmový čas ještě neplatil</dt><dd>Program počítá s místním časem podle zeměpisné délky.</dd>
  <dt>Města přecházela postupně</dt><dd>Před rokem 1900 u nás nebyl čas sjednocený. Ověřte, odkud údaj pochází.</dd>
  <dt>Letní čas za války</dt><dd>Mezi dubnem 1940 a říjnem 1945 se změny času nedaly ověřit ze zákonů.</dd>
  <dt>Slovensko 1939 až 1945</dt><dd>Údaje pro toto období nejsou ověřené.</dd>
  <dt>Jiná země před rokem 1970</dt><dd>Historie času v zahraničí se neověřovala.</dd>
</dl>
<h3>Ruční posun času</h3>
<p>Ve formuláři pod <b>Pokročilé</b>. Použijte ho, jen když víte jistě, jaký čas tehdy platil. Zadává se v hodinách od UTC: středoevropský čas je <b>1</b>, letní čas <b>2</b>, Indie <b>5.5</b>. Prázdné pole znamená automatiku.</p>`,
  },
  {
    id: "telesa", area: "calc", title: "Tělesa, uzel a Chiron",
    summary: "Co všechno program počítá a proč střední uzel.",
    keys: "planety tělesa uzel lunární dračí střední pravý chiron slunce měsíc zpětný",
    html: `
<p>Program počítá Slunce, Měsíc, osm planet včetně Pluta, Severní uzel a podle volby Chirona. K nim Ascendent a MC, pokud je znám čas narození.</p>
<h3>Severní uzel</h3>
<p>Bod, kde dráha Měsíce protíná rovinu dráhy Slunce. Program používá <b>střední uzel</b>, stejně jako výchozí nastavení Astrologu. Je to vyhlazená poloha, která se plynule pohybuje zpět. Pravý uzel, který najdete v jiných programech, se od něj liší nejvýš asi o 2°.</p>
<h3>Chiron</h3>
<p>Malé těleso mezi Saturnem a Uranem, v astrologii spojené s tématem zranění a léčení. Ve výchozím stavu je vypnutý. Zapnete ho v <a data-topic="nastaveni">Nastavení</a> a objeví se ve všech pohledech i výkladech.</p>`,
  },
  {
    id: "presnost", area: "calc", title: "Odkud jsou polohy a jak jsou přesné",
    summary: "Tropický zodiak, data NASA, porovnání s jinými programy.",
    keys: "přesnost efemeridy nasa jpl de440 tropický zodiak geocentrický swiss rozdíly jiný program 1800 2399 internet",
    html: `
<ul>
  <li><b>Pohled ze Země.</b> Polohy jsou geocentrické, tedy tak, jak je vidíme ze Země.</li>
  <li><b>Tropický zodiak.</b> 0° Berana je jarní bod, jako v západní astrologii.</li>
  <li><b>Data NASA JPL.</b> Polohy vychází z dat DE440 a Horizons, která jsou přímo v programu.</li>
  <li><b>Bez internetu.</b> Program ani seznam míst internet nepotřebují.</li>
  <li><b>Rozsah.</b> Roky 1800 až 2399.</li>
</ul>
<p>Výpočty jsou průběžně porovnávané s původním Astrologem, NASA JPL Horizons a Swiss Ephemeris.</p>
<h3>Proč jiný program ukazuje něco jiného</h3>
<ul>
  <li>Jiný <a data-topic="domy">systém domů</a>.</li>
  <li>Pravý místo středního <a data-topic="telesa">uzlu</a>.</li>
  <li>Jiné <a data-topic="aspekty">orby</a>, proto se liší seznam aspektů.</li>
  <li>Jiný posun času. Porovnejte údaj u <a data-topic="cas">času narození</a>.</li>
  <li>Sideralní zodiak, například ve védské astrologii, se od tropického liší zhruba o 24°.</li>
</ul>`,
  },

  {
    id: "nastaveni", area: "data", title: "Všechna nastavení",
    summary: "Přehled voleb a co která změní.",
    keys: "nastavení volby možnosti vedlejší chiron noční režim světlý tmavý vzhled velikost písma větší písmo zvětšit lupa čitelnost",
    html: `
<p>Nastavení otevřete v levém sloupci dole. Platí pro všechny lidi a zůstanou uložená.</p>
<dl>
  <dt>Systém domů</dt><dd>Jak se horoskop dělí na domy. Výchozí je Placidus. <a data-topic="domy">Rozdíly a porovnání</a>.</dd>
  <dt>Zobrazovat Chirona</dt><dd>Přidá Chirona do kol, tabulek, výkladů i kalendáře. <a data-topic="telesa">Více</a>.</dd>
  <dt>Zobrazovat vedlejší aspekty</dt><dd>Přidá kvinkunx, polosextil, polokvadrát a seskvikvadrát. V kole přibude čar a ve výkladu textů. <a data-topic="aspekty">Více</a>.</dd>
  <dt>Progresivní osy a domy solárním obloukem</dt><dd>Mění pohyb Ascendentu, MC a domů v progresi. <a data-topic="osy">Rozdíl obou způsobů</a>.</dd>
  <dt>Velikost písma na obrazovce</dt><dd>Běžné, Větší, Velké nebo Největší. Zvětší se celé okno, text, kolo i tlačítka, a rozložení se přizpůsobí. Změna se ukáže hned, uloží se tlačítkem Hotovo. Když okno zavřete bez Hotovo, velikost se vrátí.</dd>
  <dt>Velikost písma při tisku a v PDF</dt><dd>Menší, Běžné, Větší nebo Velké. Mění text na vytištěném listu, kolo zůstává stejně velké. Větší písmo zabere víc stránek. <a data-topic="tisk">Tisk a PDF</a>.</dd>
  <dt>Verze programu</dt><dd>Je dole v Nastavení. O nové verzi dá program vědět sám, viz <a data-topic="aktualizace">Aktualizace programu</a>.</dd>
  <dt>Denní a noční režim</dt><dd>Přepínač je nahoře vedle názvu programu. Tisk je vždy světlý.</dd>
  <dt>Záloha lidí</dt><dd>Uložení do souboru a načtení. <a data-topic="zaloha">Více</a>.</dd>
</dl>`,
  },
  {
    id: "zaloha", area: "data", title: "Záloha lidí",
    summary: "Automatické kopie, záloha do souboru a její načtení.",
    keys: "záloha kopie obnova ztráta soubor flash disk přenos jiný počítač načíst data lide.json",
    html: `
<p>Lidé jsou uložení jen v tomto počítači, nic se nikam neposílá.</p>
<h3>Automatické kopie</h3>
<p>Při první změně v každém dni si program sám uloží kopii seznamu. Drží posledních 30 kopií, starší maže. Najdete je při načítání zálohy.</p>
<h3>Záloha do souboru</h3>
<p>V Nastavení zvolte <b>Uložit zálohu do souboru</b>. Mějte ji i mimo tento počítač, například na flash disku. Automatické kopie před poruchou počítače neochrání.</p>
<h3>Načtení zálohy</h3>
<p><b>Načíst zálohu</b> jen přidá lidi, kteří v programu chybí. Nikoho nesmaže ani nezmění, jen člověku bez skupiny doplní skupinu ze zálohy. Je to proto bezpečné i opakovaně. Stejně přenesete lidi na jiný počítač.</p>`,
  },
  {
    id: "aktualizace", area: "data", title: "Aktualizace programu",
    summary: "Jak program pozná novou verzi a jak ji nainstaluje.",
    keys: "aktualizace nová verze instalace stáhnout internet github odinstalovat",
    html: `
<p>Při každém spuštění se program zeptá na stránce vydání na GitHubu, jestli je novější verze. Neposílá při tom žádné údaje o lidech ani nastavení. Bez internetu se nic neděje a program normálně funguje.</p>
<p>Když novější verze je, ukáže se v levém sloupci nahoře lišta s tlačítkem <b>Aktualizovat</b>. Program stáhne instalátor, ověří jeho podpis, zavře se, nainstaluje novou verzi a sám se znovu spustí. <b>Později</b> lištu skryje do příštího spuštění.</p>
<p>Lidé, nastavení a zálohy jsou uložené zvlášť a aktualizace je nemění. Lidi, které máte jen zobrazené, je potřeba před aktualizací uložit.</p>
<p>Program je nainstalovaný jen pro přihlášeného uživatele, bez hesla správce. Odinstalovat ho jde ve Windows v Nastavení, v části Aplikace. Lidé přitom zůstanou na disku.</p>`,
  },
  {
    id: "vyklady", area: "data", title: "Odkud jsou výklady",
    summary: "Jak texty vznikly a jak je číst.",
    keys: "výklad texty původ interpretace zdraví předpověď",
    html: `
<p>Výklady jsou vlastní české texty napsané pro tento program. Obecně známé astrologické významy jsou v nich popsané nově vlastními slovy, nejsou překladem ani převyprávěním žádného cizího textu. Pokud výklad pro nějakou kombinaci chybí, program použije krátkou větu složenou podle původního Astrologu.</p>
<p>Texty popisují <b>oblasti pozornosti a sklony</b>. Nepředpovídají události, nestanovují diagnózy a nenahrazují lékaře, právníka ani jiného odborníka. Stejná konstelace se u různých lidí projevuje různě.</p>
<p>Autoři se v detailech liší a astrologie není ověřená vědecká metoda. Berte texty jako podnět k přemýšlení.</p>`,
  },
  {
    id: "licence", area: "data", title: "Autoři a licence",
    summary: "Z čeho program vychází a pod jakými licencemi.",
    keys: "licence autoři autorská práva astrolog pullen gpl nasa jpl geonames písmo astronomicon erfa knihovny tauri zdrojový kód",
    html: `
<p>Program je svobodný software pod licencí <b>GNU GPL verze 2 nebo novější</b>. Smíte ho používat, kopírovat a dál šířit. Protože obsahuje knihovny pod licencí Apache 2.0, šíří se sestavený program podle GPL verze 3, což tato licence dovoluje. Program je bez jakékoli záruky. Zdrojový kód je veřejně na GitHubu: github.com/N4t4niel/kosmogram.</p>
<h3>Z čeho program vychází</h3>
<dl>
  <dt>Astrolog</dt><dd>Kosmogram je samostatná česká verze, ne původní program. Výpočty, tabulky a složené věty výkladu vychází z programu Astrolog 8.00, jehož autorem je Walter D. Pullen (astrolog.org). Upozornění autora je níže beze změny.</dd>
  <dt>Polohy těles</dt><dd>NASA JPL, data DE440 a Horizons.</dd>
  <dt>Precese a nutace</dt><dd>Modely IAU, koeficienty z knihovny ERFA odvozené z IAU SOFA, licence BSD.</dd>
  <dt>Místa</dt><dd>GeoNames (geonames.org), licence CC BY 4.0. Data jsou upravená: výběr obydlených míst a česká jména.</dd>
  <dt>Časová pásma</dt><dd>IANA Time Zone Database, volné dílo. Letní čas v Československu podle Sbírky zákonů.</dd>
  <dt>Symboly</dt><dd>Písmo Astronomicon, autor Roberto Corona, licence SIL OFL 1.1.</dd>
  <dt>Knihovny</dt><dd>Tauri a další knihovny jazyka Rust, hlavně pod licencemi MIT a Apache 2.0.</dd>
  <dt>Výklady a nápověda</dt><dd>Vlastní české texty, viz <a data-topic="vyklady">Odkud jsou výklady</a>.</dd>
</dl>
<p>Úplné texty všech licencí jsou ve složce <b>licenses</b> ve složce programu.</p>
<h3>Upozornění autora Astrologu</h3>
<p>Swiss Ephemeris ani PLACALC, které upozornění zmiňuje, tato verze neobsahuje. Upozornění se ale musí zachovat celé.</p>
<pre class="notice">/*
** Astrolog (Version 8.00) File: astrolog.h
**
** IMPORTANT NOTICE: Astrolog and all chart display routines and anything
** not enumerated below used in this program are Copyright (C) 1991-2026 by
** Walter D. Pullen (Astara@msn.com, http://www.astrolog.org/astrolog.htm).
** Permission is granted to freely use, modify, and distribute these
** routines provided these credits and notices remain unmodified with any
** altered or distributed versions of the program.
**
** The main ephemeris databases and calculation routines are from the
** library SWISS EPHEMERIS and are programmed and copyright 1997-2008 by
** Astrodienst AG. Use of that source code is subject to license for Swiss
** Ephemeris Free Edition at https://www.astro.com/swisseph/swephinfo_e.htm.
** This copyright notice must not be changed or removed by any user of this
** program.
**
** Additional ephemeris databases and formulas are from the calculation
** routines in the program PLACALC and are programmed and Copyright (C)
** 1989,1991,1993 by Astrodienst AG and Alois Treindl (alois@astro.ch). The
** use of that source code is subject to regulations made by Astrodienst
** Zurich, and the code is not in the public domain. This copyright notice
** must not be changed or removed by any user of this program.
**
** The original planetary calculation routines used in this program have
** been copyrighted and the initial core of this program was mostly a
** conversion to C of the routines created by James Neely as listed in
** 'Manual of Computer Programming for Astrologers', by Michael Erlewine,
** available from Matrix Software.
**
** Atlas composed using data from https://www.geonames.org/ licensed under a
** Creative Commons Attribution 4.0 License. Time zone changes composed using
** public domain TZ database: https://data.iana.org/time-zones/tz-link.html
**
** The PostScript code within the core graphics routines are programmed
** and Copyright (C) 1992-1993 by Brian D. Willoughby (brianw@sounds.wa.com).
**
** More formally: This program is free software; you can redistribute it
** and/or modify it under the terms of the GNU General Public License as
** published by the Free Software Foundation; either version 2 of the
** License, or (at your option) any later version. This program is
** distributed in the hope that it will be useful and inspiring, but
** WITHOUT ANY WARRANTY; without even the implied warranty of
** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
** General Public License for more details, a copy of which is in the
** LICENSE.HTM file included with Astrolog, and at http://www.gnu.org
**
** Initial programming 8/28-30/1991.
** X Window graphics initially programmed 10/23-29/1991.
** PostScript graphics initially programmed 11/29-30/1992.
** Last code change made 5/28/2026.
*/</pre>`,
  },
  {
    id: "slovnicek", area: "data", title: "Slovníček pojmů",
    summary: "Krátké vysvětlení odborných slov.",
    keys: "slovník pojmy význam",
    html: `
<dl>
  <dt>Ascendent</dt><dd>Stupeň znamení, který při narození vycházel na východě.</dd>
  <dt>Aspekt</dt><dd>Význačný úhel mezi dvěma tělesy.</dd>
  <dt>Dům</dt><dd>Jedna z dvanácti oblastí života v horoskopu.</dd>
  <dt>Efemeridy</dt><dd>Tabulky poloh těles v čase.</dd>
  <dt>Hrot domu</dt><dd>Začátek domu na kole.</dd>
  <dt>Kvality</dt><dd>Dělení znamení na kardinální, pevná a pohyblivá.</dd>
  <dt>MC, Medium coeli</dt><dd>Nejvyšší bod dráhy těles nad obzorem, začátek 10. domu v kvadrantových systémech.</dd>
  <dt>Orb</dt><dd>Tolerance aspektu ve stupních.</dd>
  <dt>Progrese</dt><dd>Metoda, kde jeden den po narození odpovídá jednomu roku života.</dd>
  <dt>Radix</dt><dd>Horoskop z okamžiku narození.</dd>
  <dt>Retrográdní, zpětný pohyb</dt><dd>Zdánlivý pohyb tělesa zpět při pohledu ze Země.</dd>
  <dt>Severní uzel</dt><dd>Bod, kde Měsíc přechází přes rovinu dráhy Slunce směrem na sever.</dd>
  <dt>Solární návrat</dt><dd>Okamžik, kdy se Slunce vrátí na polohu z narození.</dd>
  <dt>Solární oblouk</dt><dd>Vzdálenost, o kterou postoupilo progresivní Slunce. Asi 1° za rok.</dd>
  <dt>Synastrie</dt><dd>Porovnání dvou horoskopů.</dd>
  <dt>Tranzit</dt><dd>Aspekt tělesa na obloze k bodu radixu.</dd>
  <dt>Živly</dt><dd>Dělení znamení na ohnivá, zemská, vzdušná a vodní.</dd>
</dl>`,
  },
];
