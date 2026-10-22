# Querencia Editor

Klassischer Texteditor mit Reitern für alle, die Querencia Desktop benutzen.
Teil von [Querencia Desktop](https://querencialinux.org) –
einer klassischen Desktopumgebung im GNOME-2-Stil für Wayland (labwc, Rust,
GTK 4).
Installiert wird es zusammen mit Querencia Desktop aus der APT-Quelle;
Anleitung auf <https://querencialinux.org>.

*English summary:* A classic tabbed text editor modelled on GNOME 2's gedit
2.30, built with GTK 4 and GtkSourceView 5. Part of Querencia Desktop, a
classic GNOME-2-style desktop for Wayland. Alpha software; documentation is
in German, the user interface is available in English and German.

> **Alpha.** Prototyp: kein Drucken, keine Werkzeugleiste, keine Plugins.
> Keine Sicherheits- oder Datenverlustgarantie.

## Zweck

- Texte bearbeiten in mehreren Reitern einer Instanz (weitere Aufrufe öffnen
  Reiter), mit Menüleiste Datei · Bearbeiten · Ansicht · Suchen · Dokumente ·
  Hilfe, Zugriffstasten und gedit-2-Kürzeln.
- UTF-8 (Zeichensatz und Zeilenende geladener Dateien bleiben erhalten),
  Zeilennummern, automatische Einrückung, Klammerhervorhebung, optionaler
  Zeilenumbruch; Hervorhebung über GtkSourceView (`.md .conf .ini .json .xml
  .sh .py .rs .c` fest zugeordnet, `.txt` als reiner Text).
- Suchen (Strg+F), Weitersuchen (Strg+G), Ersetzen (Strg+H), Gehe zu Zeile
  (Strg+I); Rückfrage vor dem Schließen geänderter Dokumente.
- Oberfläche auf Englisch und Deutsch über GNU gettext (englische msgid,
  `po/de.po`); Auswahl über `LANGUAGE`/`LANG`.
- Bewusst **nicht** Teil dieses Programms: Drucken, Werkzeugleiste,
  Seitenleiste, Plugins, Rechtschreibprüfung, Sitzungswiederherstellung,
  Einstellungsdialog.

Anwendungskennung (App-ID): `org.querencia.Editor` · Binary:
`querencia-editor` · Debian-Paket: `querencia-editor`

| Datei | Inhalt |
|---|---|
| `src/` | Fenster, Reiter, Dokumente, Suchen/Ersetzen, Sprachen, Übersetzung (`i18n.rs`) |
| `po/` | Vorlage `querencia-editor.pot`, Deutsch `de.po` |
| `data/` | Desktop-Datei (aus `.desktop.in`), Programm- und Fenstersymbol (ohne 48×48), Querencia-Logo, eingebettete Ressource |
| `werkzeug/i18n.py` | Kataloge prüfen und erzeugen ohne GNU gettext (`pot`, `update`, `mo`, `desktop`, `pruefen`) |
| `docs/` | Bauen, Prüfen, Muster für die Mehrsprachigkeit (`i18n-muster.md`) |

## Stand

| | |
|---|---|
| Version | 0.2.4 (Tag `editor/0.2.4`; Info-Dialog mit Alpha-Hinweis, Website, Copyright und Lizenz, Hilfe-Menü mit Online-Hilfe und „Fehler melden“); seit 0.2.3 dunkeltauglich, eindeutige deutsche Zugriffstasten im Such-/Ersetzen-Dialog |
| Letzte Prüfung | 22.10.2026 (0.2.4): fmt und Clippy sauber, 8 Unit-Tests (1 GUI-Test `#[ignore]`), `i18n.py pruefen` (80 Texte, Zugriffstasten je Menü), 10 Tests `werkzeug/test_i18n.py`; isoliert Deutsch und Englisch bei 1024×768: Online-Hilfe, „Fehler melden“ (je über die Zugriffstaste) und Info-Dialog. 21.10.2026 (0.2.3): isoliert 19 Läufe hell/dunkel Deutsch und Englisch bei 1024×768 und 1920×1080 |
| Bekannte Lücken | siehe „Zweck“; die isolierte GUI-Prüfung liegt noch nicht in diesem Repo ([docs/testing.md](docs/testing.md)); MetaInfo ohne Übersetzung |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, Rust 1.93.1 |
| Nicht getestet | andere Distributionen, HiDPI, Mehrmonitor |

Änderungen: [CHANGELOG.md](CHANGELOG.md).

## Bauen

Voraussetzungen (Ubuntu 26.04):

```bash
sudo apt install build-essential pkg-config rustc cargo libgtk-4-dev libgtksourceview-5-dev
```

```bash
cargo build --release --locked
```

Das Ergebnis liegt unter `target/release/querencia-editor`. Übersetzungen,
Ablage der Kataloge und Symbolgrößen: [docs/building.md](docs/building.md).

## Prüfen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 werkzeug/i18n.py pruefen
```

Diese Befehle laufen ohne Bildschirm; die ersten drei sind auch die CI
(`.github/workflows/ci.yml`). Die **GUI-Prüfung** läuft in einer
abgeschotteten Sitzung (eigener labwc ohne Bildschirm, eigenes `HOME` und
`XDG_RUNTIME_DIR`), niemals in der laufenden Desktopsitzung; ihr Werkzeug
liegt noch außerhalb dieses Repos.

Fälle und Ablauf: [docs/testing.md](docs/testing.md). Arbeitsweise insgesamt:
[CONTRIBUTING.md](CONTRIBUTING.md).

## Vorbild und Abgrenzung

Vorbild für Bedienung und Aussehen ist **gedit 2.30 aus GNOME 2.30**.
Querencia übernimmt davon die Bedienideen (Menüs, Kürzel, Anordnung) und den
englischen Wortschatz der Menüs, **keinen Quellcode, keine Grafiken**; aus
Pluma, Xed oder Mousepad wurde kein Code und kein Text übernommen. Das Projekt
ist mit gedit, GNOME und deren Autoren weder verbunden noch von ihnen
unterstützt.

Abweichungen vom Vorbild und Dinge, die bewusst fehlen:

- GTK 4 und GtkSourceView 5 statt GTK 2, Fensterrahmen vom Compositor
  (labwc), kein libadwaita.
- Kein Drucken, keine Plugins, keine Seitenleiste.

Fremdbestandteile: nur Rust-Abhängigkeiten (gtk4-rs, sourceview5, gettext-rs
und Umfeld) unter MIT, MIT OR Apache-2.0, Apache-2.0, Apache-2.0 WITH
LLVM-exception, Unicode-3.0 und Unlicense OR MIT; Liste mit
`cargo metadata --locked`. Dynamisch gelinkt: GTK 4, GtkSourceView 5 und das
gettext der glibc.

## Lizenz

Programm: GPL-3.0-or-later, siehe [LICENSE](LICENSE). Copyright 2026
Querencia Desktop contributors; Urheberhinweis in
[COPYRIGHT](COPYRIGHT). Das Querencia-Logo in `data/logo/` ist unverändert
übernommen (Prüfsummen in `data/logo/SHA256SUMS.herkunft`).

Name und Logo „Querencia“ sind keine freien Inhalte: [MARKE.md](MARKE.md).

## Fehler melden

Fehler, Wünsche und Fragen zu Querencia gehen an die gemeinsame Meldestelle:
<https://github.com/querencia-desktop/querencia/issues/new/choose>.
Sicherheitslücken bitte nicht öffentlich melden, siehe [SECURITY.md](SECURITY.md).
