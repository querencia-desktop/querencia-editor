# Mehrsprachigkeit mit gettext – Muster für Querencia-Programme

Vorbild ist der Querencia Editor 0.2.0 (G2-AC, 19.10.2026). Dieses Dokument
beschreibt Schritt für Schritt, wie ein weiteres Querencia-Programm (Rust,
GTK 4 über gtk-rs 0.11, ohne libadwaita) übersetzbar wird. Je Programm
reicht danach ein kleiner Auftrag: „i18n nach `docs/i18n-muster.md` des
Editors, Zielsprachen de (+ en als msgid)“.

Entscheidungen (19.10.2026):

- **GNU gettext** mit `.po`/`.mo` wie GNOME 2 und Debian. Ausnahme: Files
  (Strata-Fork) bleibt bei `rust-i18n`.
- **msgid englisch**, Deutsch kommt aus `po/de.po`. Gründe: Programme ohne
  Katalog für die Sprache des Benutzers (fr, es, C/POSIX) fallen auf
  Englisch zurück, nicht auf Deutsch; GTK, GLib, GtkSourceView und
  Desktop-Dateien sind ebenso aufgebaut; Übersetzer und Werkzeuge
  (Weblate, Launchpad, Debian) erwarten Englisch als Quelle; Pluralformen
  sind auf englische Quelle ausgelegt. Kommentare, Bezeichner, Doku und
  Commits bleiben deutsch.
- **Preis:** Deutsch hängt an `de.mo`. Deshalb prüfen Test, Werkzeug und
  Integrationsskript, dass der deutsche Katalog vollständig ist und in der
  Ablage liegt.
- **Sprachwahl** wie bei gettext üblich: `LANGUAGE` vor `LC_ALL`,
  `LC_MESSAGES`, `LANG`. Die Querencia-Sitzung läuft heute mit
  `LANG=en_US.UTF-8` und `LANGUAGE=de` (labwc `environment`). Deutsch wird
  also über `LANGUAGE` gewählt. Ein Einstellungsbereich „Sprache“ folgt
  später (QD-046-Reihe).

## 1. Crate

```toml
[dependencies]
# Übersetzungen über GNU gettext (po/, .mo unter share/locale). Feature
# gettext-system: das gettext der glibc statt eines mitgebauten libintl.
gettext-rs = { version = "0.8", features = ["gettext-system"] }
```

- `gettext-rs` 0.8.0 / `gettext-sys` 0.27.0 passen zu gtk-rs 0.11 (keine
  gemeinsamen Abhängigkeiten, keine Versionskonflikte).
- **Immer `gettext-system`.** Ohne das Feature baut `gettext-sys` GNU
  gettext aus dem Quelltext (autotools, Minuten, eigenes libintl). Mit dem
  Feature nutzt es das gettext der glibc und linkt nichts zusätzlich.
- Neue Abhängigkeiten im Lock: `gettext-rs`, `gettext-sys`,
  `locale_config` (zieht `regex`, `lazy_static`; `objc*`/`winapi*` nur für
  andere Plattformen, werden auf Linux nicht gebaut). Release-Binary des
  Editors: 614 344 → 642 008 Byte (+27 KB, davon ein Teil die neuen Texte).
- `setlocale` ist in gettext-rs 0.8 `unsafe`. Unsere Programme haben
  `unsafe_code = "forbid"`. Das ist kein Problem: **GTK ruft beim Start
  selbst `setlocale(LC_ALL, "")`** (in `gtk_init`, also beim `startup` der
  `gtk::Application`). Alle Texte entstehen erst danach. Bei Programmen ohne
  GTK-Start vor den ersten Texten (CLI-Teile, Diagnosen) gelten die Texte
  als Log und bleiben unübersetzt. Alternativ bekommt nur das Modul `i18n`
  ein `#[allow(unsafe_code)]`, und die Lints stehen auf `deny` statt
  `forbid`. Das braucht eine Begründung im Auftrag.
- Nicht verwendet: `glib::dgettext` (es fehlt `bindtextdomain`), `tr`/`xtr`
  (zusätzliche Makros, `xtr` ist kaum gepflegt), `rust-i18n`/Fluent (kein
  gettext, keine Debian-Werkzeuge).

## 2. Modul `src/i18n.rs`

Vom Editor übernehmen und `DOMAIN`, den Variablennamen
`QUERENCIA_EDITOR_LOCALEDIR` und die Ablage-Beschreibung anpassen. Kern:

```rust
pub use gettextrs::{gettext, ngettext, pgettext};
pub const DOMAIN: &str = "querencia-editor";          // = Name der .mo

pub fn init() {                                        // vor Application::run
    let dir = locale_dir();
    let _ = bindtextdomain(DOMAIN, dir.as_path());
    let _ = bind_textdomain_codeset(DOMAIN, "UTF-8");
    let _ = textdomain(DOMAIN);
}
```

**Suchreihenfolge der Kataloge** (`locale_dir`):

1. Umgebungsvariable `QUERENCIA_<PROGRAMM>_LOCALEDIR` (Tests, Entwicklung).
2. `<Ordner der Binary>/share/locale`. Das ist die unveränderliche Ablage
   `~/opt/querencia/<komp>/<sha16>/`. `current_exe()` löst den Link in
   `~/opt/querencia/bin/` auf.
3. `<Ordner der Binary>/../share/locale`, also das Präfix:
   `/usr/bin` → `/usr/share/locale`, `/usr/local/bin` →
   `/usr/local/share/locale`.
4. `/usr/share/locale`.

`main.rs`: `mod i18n;` und als erste Zeile in `main()` `i18n::init();`.

**Platzhalter:** `gettext_f("Saved “{name}”.", &[("name", &name)])` und
`ngettext_f(singular, plural, n, &[…])`. `{n}` setzt `ngettext_f` selbst ein.
`fill` ersetzt in einem Durchgang: Werte mit `{…}` darin werden nicht noch
einmal ersetzt. Nie `format!` auf übersetzten Text anwenden, und nie Sätze
aus Teilen zusammenkleben. Die Wortstellung gehört dem Übersetzer.

## 3. Texte im Code umstellen

| vorher | nachher |
|---|---|
| `Label::new(Some("Normaler Text"))` | `Label::new(Some(&gettext("Plain Text")))` |
| `Button::with_mnemonic("_Abbrechen")` | `Button::with_mnemonic(&gettext("_Cancel"))` |
| `format!("„{name}“ gespeichert.")` | `gettext_f("Saved “{name}”.", &[("name", &name)])` |
| `match n { 1 => "Ein …", n => format!("{n} …") }` | `ngettext_f("Replaced one occurrence.", "Replaced {n} occurrences.", n, &[])` |
| Menü-Tupel `("_Neu", "win.new")` | `(gettext("_New"), "win.new")`, Hilfsfunktion nimmt `&[(String, &str)]` |
| gleiches Englisch, andere deutsche Zugriffstaste | `pgettext("File menu", "_Close")` (→ „S_chließen“) neben `gettext("_Close")` (→ „_Schließen“) |

Regeln:

- **Nur Literale** als msgid. `gettext(variable)` findet kein Werkzeug.
- **Übersetzerhinweis:** `// TRANSLATORS: …` direkt über dem Aufruf oder im
  Aufruf vor dem Literal. Pflicht für Abkürzungen (INS/OVR), knappe
  Statusleistentexte und Knöpfe, deren Zugriffstasten sich in einem Dialog
  nicht überschneiden dürfen.
- **Wortschatz GNOME 2 / gedit 2.30:** File, Edit, View, Search, Documents,
  Help, About, Preferences, Quit, Close, Save As…, Find…, Find Next, Find
  Previous, Replace…, Go to Line…, Unsaved Document N, Close without Saving,
  Plain Text, Ln/Col, INS/OVR. Englische Typografie: `…` ohne Leerzeichen,
  Anführungszeichen “ ”, Apostroph ’. Im deutschen Katalog bleiben „ “ und
  „ …“ mit Leerzeichen wie bisher.
- **Zugriffstasten (`_`):** je Menü und je Dialog eindeutig, für jede
  Sprache getrennt prüfen. Englisch nach gedit (`Cu_t`, `Find Ne_xt`,
  `Wra_p around`). Deutsch exakt wie bisher. Testskripte, die Alt+Taste
  drücken, brauchen je Sprache eine eigene Taste (Editor: „Ohne Speichern
  schließen“ Alt+O, „Close without Saving“ Alt+W).
- **Nicht übersetzen:** Programmname („Querencia Editor“), Fenstertitel-
  Muster (`{name} – Querencia Editor`), App-IDs, Aktionsnamen,
  Tastenkürzel (`<Control>s`), CSS-Klassen, Log- und Diagnoseausgaben auf
  stderr (`eprintln!`), D-Bus-Namen.
- **Fremde Texte:** GTK (Dateidialog, Kontextmenü), GLib-Fehlermeldungen
  (`err.message()`) und GtkSourceView-Sprachnamen übersetzen deren eigene
  Domänen (`gtk40`, `glib20`, `gtksourceview-5`) über dieselbe Locale.
  Nichts doppelt übersetzen.

## 4. Werkzeug und po-Pflege

Das Repo bringt `werkzeug/i18n.py` mit: reines Python, keine
Abhängigkeiten. Es entstand, als GNU gettext auf der VM noch fehlte. Seit
20.10. ist gettext 0.23.2 installiert; dessen `xgettext` kann aber erst ab
0.24 Rust lesen. Deshalb bleibt `i18n.py pot` der Weg zur `.pot`. Abgleich
mit gettext 0.23.2 (G2-AC):

- `msgfmt -c po/de.po` ist fehlerfrei.
- Die `.mo` aus `i18n.py` und aus `msgfmt` sind inhaltsgleich (`msgunfmt`).
  Sie unterscheiden sich nur in der Hashtabelle und einem leeren Kopffeld.
- Die Extraktion entspricht `xgettext -L C` mit denselben Schlüsselwörtern.
  Ausnahme ist die Rust-Fortsetzungszeile `"…\n\`. `-L C` behält dort
  fälschlich die Einrückung, `i18n.py` liest sie richtig.
- `i18n.py desktop` ist byte-gleich mit `msgfmt --desktop`.
- `msgcmp` meldet für `de.po` gegen `.pot` keine Abweichung.

Kopieren und `DOMAIN`/`PACKAGE` oben anpassen.

```bash
python3 werkzeug/i18n.py pot        # po/<domäne>.pot aus src/**/*.rs und data/*.desktop.in
python3 werkzeug/i18n.py update     # po/<lang>.po nachziehen (neue leer, alte als #~)
$EDITOR po/de.po                    # übersetzen; fuzzy entfernen
python3 werkzeug/i18n.py desktop data/<id>.desktop.in data/<id>.desktop
python3 werkzeug/i18n.py pruefen    # PASS/FAIL, siehe unten
python3 werkzeug/i18n.py mo <ziel>  # <ziel>/<lang>/LC_MESSAGES/<domäne>.mo
```

- `po/LINGUAS` listet die Sprachen (heute `de`). Englisch braucht keinen
  Katalog, weil die msgid englisch ist.
- `pruefen` meldet FAIL bei veralteter .pot, fehlenden, leeren oder
  fuzzy-Einträgen, abweichenden Platzhaltern (`{name}`), fehlender oder
  doppelter Zugriffstaste und veralteter erzeugter Desktop-Datei.
  `msgstr[0]` darf `{n}` weglassen („Ein Vorkommen ersetzt.“).
- Mit GNU gettext ≥ 0.24 (Rust-Unterstützung) gilt dasselbe so:
  ```bash
  xgettext -L Rust --from-code=UTF-8 --add-comments=TRANSLATORS \
    -k -kgettext -kgettext_f -kngettext:1,2 -kngettext_f:1,2 -kpgettext:1c,2 \
    -o po/querencia-editor.pot src/*.rs
  msgmerge -U po/de.po po/querencia-editor.pot
  msgfmt -c -o share/locale/de/LC_MESSAGES/querencia-editor.mo po/de.po
  msgfmt --desktop -d po --template data/org.querencia.Editor.desktop.in -o data/org.querencia.Editor.desktop
  ```
  `-k` schaltet die eingebauten Rust-Schlüsselwörter ab. Sonst würde
  `gettext_f` je nach Version doppelt erfasst. Für Pakete genügen
  `msgfmt` und `msgfmt --desktop` aus gettext 0.23 (Debian/Ubuntu-Paket
  `gettext`). Die `.pot` liegt erzeugt im Repo.
- **Plural:** Kopf `Plural-Forms: nplurals=2; plural=(n != 1);` für de/en.
  Für weitere Sprachen die GNU-Tabelle nehmen (fr: `n > 1`; pl, ru, cs
  haben 3 Formen). `i18n.py update` setzt de/en/fr selbst. Andere Sprachen
  von Hand eintragen.
- **Kontext** (`msgctxt`) nur, wenn dasselbe Englisch in einer Sprache
  verschieden heißen muss.
- **Test in Rust** (`german_catalog_is_complete`): liest `.pot` und
  `de.po` per `include_str!` und scheitert an fehlenden, leeren oder
  fuzzy-Einträgen. Das hält `cargo test` auch ohne Python ehrlich.

## 5. GTK4-Builder-Dateien (.ui/.blp)

Heute hat **kein** Querencia-Programm `.ui`- oder Blueprint-Dateien; alle
Oberflächen entstehen im Code. Falls doch welche dazukommen:

- In der `.ui`: `<interface domain="querencia-xyz">` und je Text
  `<property name="label" translatable="yes" context="…" comments="…">_Open</property>`.
- `gtk::Builder` übersetzt beim Laden über die Domäne. `textdomain()` muss
  vorher gesetzt sein, also `i18n::init()` vor dem Laden.
- Als GResource eingebettete `.ui` bleiben übersetzbar: Die Übersetzung
  geschieht zur Laufzeit, nicht beim Kompilieren.
- Extraktion: `xgettext -L Glade` bzw. `blueprint-compiler` erzeugt `.ui`.
  `werkzeug/i18n.py` kann `.ui` heute **nicht** lesen. Erweitern: Elemente
  mit `translatable="yes"` sammeln.
- Menüs aus `gio::Menu` im Code (wie im Editor) brauchen nichts davon.

## 6. Desktop-Datei und MetaInfo

- Vorlage `data/<id>.desktop.in` mit **englischen** `Name`, `GenericName`,
  `Comment`, `Keywords`. Die Schlüssel landen mit Hinweis in der `.pot`.
  `i18n.py desktop` (bzw. `msgfmt --desktop`) schreibt
  `data/<id>.desktop` mit `Name[de]=…` usw. Beide liegen im Repo. `pruefen`
  achtet darauf, dass die erzeugte Datei aktuell ist.
- `Keywords` als Ganzes übersetzen, mit abschließendem `;`.
- Die Integrationsvorlage entsteht aus der erzeugten Datei; nur
  `Exec`/`TryExec` zeigen auf die Ablage. `desktop-file-validate` muss
  schweigen.
- **MetaInfo** (`*.metainfo.xml`): Heute liegen die 14 Dateien nur unter
  `~/.local/share/metainfo/` (Querencia Software), **nicht** in den
  Programm-Repos. Muster, wenn sie ins Repo kommen: englische Quelle mit
  `<name>`, `<summary>`, `<description>`. Übersetzung über `itstool`
  bzw. `msgfmt --xml` (gettext ≥ 0.19.7). `i18n.py` kann das noch nicht.
  Bis dahin bleiben MetaInfo-Dateien zweisprachig von Hand
  (`<summary xml:lang="de">`).

## 7. Ablage, Integration, Paket

- **Ablage** `~/opt/querencia/<komp>/<sha16>/` (Verzeichnisse 0555):
  ```
  querencia-xyz                                   0555
  share/locale/de/LC_MESSAGES/querencia-xyz.mo    0444
  SHA256SUMS                                      0444  (der beiden Dateien)
  ```
  Die Binary findet die Kataloge über ihren eigenen Pfad (siehe §2). Der
  Link in `~/opt/querencia/bin/` bleibt wie bisher.
- **Integrationsskript** nach `editor-i18n-integrieren.sh` (G2-AC, Muster
  G2-V `programm-integrieren.sh`):
  - Die Ablage entsteht vollständig unter einem Hilfsnamen, wird geprüft
    (genau drei Dateien, Prüfsummen, schreibgeschützt) und erst dann in
    einem Schritt umbenannt. Es gibt also nie eine Binary ohne Katalog.
  - `invarianten` verweigert, wenn der deutsche Katalog in der Vorlage
    fehlt.
  - `zustand_p` gilt nur dann als „neu“, wenn die Ablage vollständig ist.
- **Paket** (`.deb`, später): `/usr/bin/querencia-xyz`,
  `/usr/share/locale/<lang>/LC_MESSAGES/querencia-xyz.mo`, Desktop-Datei
  mit `[de]`-Zeilen. Build-Abhängigkeit `gettext` (für `msgfmt -c`),
  Laufzeit keine (glibc). Nach Debian-Brauch erzeugt `debian/rules` die
  `.mo` aus `po/`; `.mo` werden nicht ins Repo eingecheckt.
- **Sitzung:** `LANGUAGE` steuert die Sprache. Englisch prüfen mit
  `LANGUAGE=en querencia-xyz`, Deutsch mit `LANGUAGE=de` (bei
  `LANG=en_US.UTF-8` wie live). Achtung: glibc beachtet `LANGUAGE` nicht,
  solange die Locale `C`/`POSIX` ist.

## 8. Prüfung je Programm

1. `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`,
   `cargo test --locked` (mit Katalogtest).
2. `python3 werkzeug/i18n.py pruefen` → PASS.
3. Deutsch byte-gleich zum alten Stand: alle deutschen Literale aus dem
   alten Commit müssen in `po/de.po` stehen. Abweichungen sind nur
   umbenannte Platzhalter. Der Editor hat das mit einem Einzeiler über
   `git show <alt>:src/*.rs` geprüft.
4. GUI isoliert (headless labwc, Muster `~/projects/querencia-editor-i18n/werkzeug/`):
   je `LANGUAGE=de` und `LANGUAGE=en`, 1024×768 und 1920×1080. Belegbilder
   von Menüs, einem Dialog und der Statusleiste. In der de-Sitzung
   `/proc/<pid>/maps` auf `…/share/locale/de/LC_MESSAGES/<domäne>.mo`
   prüfen (glibc blendet den Katalog ein). In der en-Sitzung darf er nicht
   auftauchen.
5. Bei 1920×1080 sind die Wartezeiten im Szenario zu verlängern. Der
   Software-Renderer zeichnet dort langsamer; im ersten Lauf führte das zu
   veralteten Bildern und Tasten im falschen Feld.

## 9. Fallen

- `gettext-sys` ohne `gettext-system` baut minutenlang GNU gettext mit.
- `setlocale` ist `unsafe`, GTK erledigt es (siehe §1).
- Gleiches Englisch, verschiedene deutsche Zugriffstasten → `pgettext`.
  `_Close` hat im Editor drei Kontexte.
- Text aus `include_str!`/`concat!` oder Zeilenfortsetzung (`"…\n\`)
  erkennt `i18n.py`, `xgettext` ebenfalls. `format!`-Ketten nicht.
- `LANGUAGE=de` bei `LANG=C` wirkt nicht: glibc ignoriert `LANGUAGE` in der
  C-Locale.
- Die Nutzerordner im GTK-Dateidialog („Bilder“, „Musik“) sind
  XDG-Verzeichnisnamen, keine Übersetzung. Sie bleiben in der englischen
  Oberfläche deutsch.
