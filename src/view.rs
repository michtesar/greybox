use raylib::ffi::Vector2;

/* 
* AffineTransformationView
*
* Similary transformation view for scene to screen
* and screen to scene. It is asymetrical transformation
* as unprejected input can go out of the viewport, which
* is considered as invalid location (altough the algoritm)
* can deliver the coordinates but in negative space.
*
* We take use virtual resolution to fit the view port of the
* scene space to the screen space by fit (object-fit in CSS)
* where we can have at least one bars on the sides (pillarbox)
* or in vertical order (please fix this I forgot the name for it).
*/
#[derive(Debug)]
struct AffineTransformationView {
    scale: f32,
    offset: Vector2,
}

/*
*Dobře, že se ptáš před dopsáním — ušetřilo ti to přepisování. Verdikt na high-level: tvar je přebujelý a drží špatný stav. Sedm metod tam, kde plán chce dvě, a ve struktuře je přesně to, co jsme se před dvěma zprávami shodli neukládat.

Po řadě, od nejdůležitějšího.

1. Struktura drží špatná pole (ř. 19–20)

scene_resolution: Vector2,
window_resolution: Vector2,

To je ta varianta, kterou jsme zamítli. Uvnitř máš být scale: f32 a offset: Vector2 — spočítané jednou v new.

Rozdíl není kosmetický:

- Rozměr okna přežívá uvnitř typu. Invariant „herní kód nikdy nevidí velikost okna" je tím děravý — view.window_resolution ho vidí.
- Každé volání scene_to_screen musí scale a offset dopočítat znovu. To jsou dvě dělení a min na každý bod a každý snímek.
- A hlavně: ta dvě odvozená čísla jsou to jediné, co transformace je. Kdybys je uložil, typ je hotový a metody jsou násobení a sčítání. Když uložíš vstupy, musíš jejich zpracování opakovat všude.

new bere rozměry okna, spolkne je a ven pustí jen scale a offset. Konstruktor je hranice, kde screen space umírá.

2. Oba veřejné směry máš prohozené (ř. 43, 48)

pub fn screen_to_scene() -> Vector2 { }
pub fn scene_to_screen() -> Option<Vector2> { }

Před dvěma zprávami jsi to měl správně. screen_to_scene je inverzní směr, ten vrací Option (klik na pruh). scene_to_screen je forward, ten Option nepotřebuje — kreslení mimo plátno je legitimní.

Komentáře nad nimi to mají prohozené taky („Public forward transformation" nad inverzní metodou). A nepíšou pravdu ani v druhé půlce — „with ints", přitom tam žádné inty nejsou.

Tohle je ta nejdražší chyba z celého M1, protože nespadne. Obě metody mají stejný tvar vstupu i výstupu, takže kompilátor je zaměnit nechá a projeví se to až v M2 posunutými souřadnicemi. Přesně proto jsem u pojmenování trval na scene_to_screen / screen_to_scene místo to_screen / to_scene — a i tak jsi to prohodil, což ti říká, jak snadné to je. Round-trip test z plánu je tvoje pojistka; napiš ho hned.

3. Pomocné metody nemají &self (ř. 28–40)

fn forward_tranform() -> Vector2 {}

Tahle funkce nemá self ani argument. Nemá z čeho a co transformovat — je to asociovaná funkce, ne metoda. Volala by se AffineTransformationView::forward_transform() a neměla by přístup k ničemu.

Rozdíl fn f() vs fn f(&self) je v Rustu zásadní a je to přesně ta věc, kterou při psaní signatur napřed přehlédneš. Rozmysli si u každé z těch sedmi: potřebuje instanci, nebo je to čistá funkce vstupů?

(A tranform → transform, překlep.)

4. Metod je moc, a většina z nich nemá být metoda

Plán: „a small value type holding scale and offset, with a method each way." Dvě metody. Ty máš sedm.

Projdi si je s otázkou „kdy se tohle volá?":

- get_scale_factor, get_uniform_scale, get_offsets — volají se jen z new, jednou. Když výsledek uložíš (bod 1), nikdo jiný je nepotřebuje. Krok konstrukce, ne API.
- forward_transform / inverse_transform vs scene_to_screen / screen_to_scene — to jsou dvě jména pro totéž. Dvojice privátní + veřejná metoda dává smysl, když se mezi nimi něco děje (validace, převod typů). Tady se neděje nic. Jedna vrstva, ne dvě.
- is_in_viewport — sám u ní píšeš „Maybe not needed???". Tvoje tušení je správné: je to jediná podmínka uvnitř screen_to_scene. Veřejná být rozhodně nemá.

Cílový tvar jsou dvě pole a tři funkce (new + dva směry). Jestli ti to přijde málo, tak je to tím, že ten problém opravdu je takhle malý — celá jeho obtížnost je v tom nesplést si směr, ne v množství kódu.

5. get_scale_factor(resolution: Vector2) -> f32 nedává smysl


Souvisí s tím ř. 30: resolutions: []Vector2 není platný Rust — pole se píše [Vector2; 2], slice &[Vector2]. Ale to je vedlejší; horší je, že obojí je špatný nápad. Máš dvě konkrétní pojmenované věci (okno, virtuální rozlišení), ne kolekci, kde na pořadí prvků nezáleží. Kolekce dvou prvků, které nejsou zaměnitelné, je vždycky varovný signál.

6. Jméno typu

AffineTransformationView pojmenovává mechanismus, ne věc. Pojmenovat typ podle jeho matematické kategorie je jako pojmenovat Player třeba MutableStateMachine.

Navíc view::AffineTransformationView kokrhá — modul už říká „view". Plán mluví o „a small value type"; nabízí se View nebo Viewport. „Affine" patří do doc komentáře, ne do jména.

Drobnosti

- ř. 1: use raylib::ffi::Vector2; — úplně stejná věc jako ten ffi::Rectangle minule. prelude::Vector2 je totéž a signalizuje správně.
- ř. 3–16: /* */ není doc komentář. Rust používá /// (nad položkou) a //! (pro modul). Tvůj blok se do cargo doc nedostane a — což je teď zajímavější — jeho příklady nikdy neprojdou doctesty, které ti od M0 běží.
- get_ prefix: v Rustu se nepoužívá. Getter se jmenuje scale(), ne get_scale(). (Konvence z API Guidelines; get je vyhrazené pro fallible indexovaný přístup jako slice::get.)
- struct je privátní (ř. 18), ale metody uvnitř jsou pub. Do main.rs se ten typ nedostane. Tvůj pub mod view; v lib.rs sám o sobě nestačí — stejná lekce o viditelnosti jako v M0, jen o patro níž.
- Zapomněl jsi jméno: pruhy nahoře a dole = letterbox. Po stranách = pillarbox, to jsi měl.

Jedna věc, kterou nechám na tobě

Má new brát virtuální rozlišení jako argument, nebo si sáhnout na konstantu?

Za argument: testovatelnost — plán říká, že testy jsou „pure maths over (window size, virtual size)", tedy počítá s oběma jako se vstupy.
Proti: čím víc míst může virtuální rozlišení určit, tím slabší je invariant, že je jedno a pevné.

Rozhodni a řekni mi proč. Pak to dopiš — a začni round-trip testem, dřív než ty dva směry stihneš prohodit potřetí.
*/
impl AffineTransformationView {
   pub fn new(window_resolution: Vector2) -> Self {
    let scene_resolution = Vector2{640, 480}
    }
}
