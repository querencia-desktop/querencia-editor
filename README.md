# Querencia Editor

Klassischer Texteditor mit Reitern für Querencia Desktop nach dem Vorbild von
gedit 2.30 (GNOME 2), geschrieben in Rust mit GTK 4 und GtkSourceView 5 ohne
libadwaita. Teil von Querencia Desktop – einer klassischen Desktopumgebung im
GNOME-2-Stil für Wayland (labwc).

*English summary:* A classic tabbed text editor modelled on GNOME 2's gedit
2.30, built with GTK 4 and GtkSourceView 5. Part of Querencia Desktop. Alpha
software; documentation is in German, the user interface is available in
English and German.

> **Alpha.** Kein Drucken, keine Werkzeugleiste, keine Plugins. Keine
> Sicherheits- oder Datenverlustgarantie.

## Zweck

- Texte bearbeiten in mehreren Reitern einer Instanz (weitere Aufrufe öffnen
  Reiter), mit Menüleiste Datei · Bearbeiten · Ansicht · Suchen · Dokumente ·
  Hilfe, Zugriffstasten und gedit-2-Kürzeln.
- UTF-8 (Zeichensatz und Zeilenende geladener Dateien bleiben erhalten),
  Zeilennummern, automatische Einrückung, Klammerhervorhebung, optionaler
  Zeilenumbruch; Hervorhebung über GtkSourceView (`.md .conf .ini .json .xml
  .sh .py .rs .c` fest zugeordnet, `.txt` als reiner Text).
- Suchen (Strg+F), Weitersuchen (Strg+G), Rückwärts (Umschalt+Strg+G),
  Ersetzen (Strg+H), Gehe zu Zeile (Strg+I); Rückfrage vor dem Schließen
  geänderter Dokumente.
- Oberfläche Englisch und Deutsch über GNU gettext (englische msgid,
  `po/de.po`); Auswahl über `LANGUAGE`/`LANG`.
- Bewusst **nicht** Teil dieses Programms: Drucken, Werkzeugleiste,
  Seitenleiste, Plugins, Rechtschreibprüfung, Sitzungswiederherstellung,
  Einstellungsdialog.

App-ID `org.querencia.Editor` · Binary `querencia-editor` · Debian-Paket:
noch keins in diesem Repo (Paketierung im Packaging-Repo).

## Stand

| | |
|---|---|
| Version | 0.2.4 (Tag `editor/0.2.4`; Info-Dialog mit Alpha-Hinweis, Website, Copyright und Lizenz, Hilfe-Menü mit Online-Hilfe und Fehler melden) |
| Auf der Entwicklungs-VM | live ist 0.2.3 (seit 21.10.2026); 0.2.4 wartet auf Freigabe |
| Letzte Prüfung | 21.10.2026: 7 Unit-Tests (1 GUI-Test `#[ignore]`), `i18n.py pruefen` (75 Texte, Zugriffstasten je Dialog), 10 Tests `werkzeug/test_i18n.py`; isoliert 19 Läufe hell/dunkel Deutsch und Englisch bei 1024×768 und 1920×1080, hell pixelgleich zu 0.2.2 |
| Bekannte Lücken | isolierte GUI-Prüfung liegt noch nicht in diesem Repo ([docs/testing.md](docs/testing.md)); MetaInfo ohne Übersetzung |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, Rust 1.93.1 |

Änderungen je Version: [CHANGELOG.md](CHANGELOG.md).

## Bauen und Prüfen

`cargo build --release --locked`, danach `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` und `python3 werkzeug/i18n.py pruefen`.
Voraussetzungen, Übersetzungskataloge und die isolierte GUI-Prüfung: [docs/building.md](docs/building.md), [docs/testing.md](docs/testing.md), Mehrsprachigkeit [docs/i18n-muster.md](docs/i18n-muster.md).

## Lizenz

Programm: GPL-3.0-or-later, siehe [LICENSE](LICENSE) (reiner Lizenztext).
Copyright 2026 Querencia Desktop contributors; Urheberhinweis in
[COPYRIGHT](COPYRIGHT). Eigener Code; aus gedit, Pluma, Xed oder Mousepad
wurde kein Code und kein Text übernommen. Das Querencia-Logo in `data/logo/`
ist unverändert übernommen (Prüfsummen in `data/logo/SHA256SUMS.herkunft`).
Name und Logo „Querencia“ sind keine freien Inhalte (Markenregel).

