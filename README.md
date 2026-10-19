# Querencia Editor (Prototyp)

Klassischer Texteditor für Querencia Desktop nach dem Vorbild von gedit 2.30
(GNOME 2), geschrieben in Rust mit GTK 4 und GtkSourceView 5 – derselbe
Stack wie Panel, Desktop, Terminal und Screenshot.

> **Status: isolierter Prototyp (QE-01).** Nicht installiert, nicht in die
> Sitzung eingebunden, keine Dateizuordnungen. Wartet auf Freigabe.

## Funktionen

| | |
|---|---|
| Fenster | eine Instanz (`org.querencia.Editor`), weitere Aufrufe öffnen Reiter; **keine eigene Titelleiste** – labwc zeichnet die klassische Titelleiste mit dem offiziellen Querencia-Logo C2 |
| Menüleiste | Datei · Bearbeiten · Ansicht · Suchen · Dokumente · Hilfe, mit Zugriffstasten und gedit-2-Kürzeln |
| Reiter | mehrere Dokumente, Schließknopf je Reiter, verschiebbar; „*“ bei Änderungen |
| Text | UTF-8 (Zeichensatz und Zeilenende geladener Dateien bleiben erhalten), Zeilennummern, automatische Einrückung, Klammerhervorhebung, optionaler Zeilenumbruch |
| Hervorhebung | `.md .conf .ini .json .xml .sh .py .rs .c` fest zugeordnet, `.txt` reiner Text, alles andere nach GtkSourceView; Markdown als bearbeitbarer Text |
| Suchen | Suchen (Strg+F), Weitersuchen (Strg+G), Rückwärts (Umschalt+Strg+G), Ersetzen (Strg+H) mit Groß-/Kleinschreibung, ganzen Wörtern, Umbruch; Gehe zu Zeile (Strg+I) |
| Schutz | Rückfrage vor dem Schließen geänderter Dokumente (Reiter, Fenster, Beenden): Speichern / Abbrechen / Ohne Speichern schließen |
| Sprache | Deutsch und Englisch über GNU gettext (englische msgid, `po/de.po`); Auswahl wie üblich über `LANGUAGE`/`LANG` |

Nicht vorhanden: Drucken, Werkzeugleiste, Seitenleiste, Plugins,
Rechtschreibprüfung, Sitzungswiederherstellung, Einstellungsdialog.

## Bauen, testen, übersetzen

[BUILDING.md](BUILDING.md), [TESTING.md](TESTING.md). Übersetzungen und das
Muster für die anderen Querencia-Programme: [docs/i18n-muster.md](docs/i18n-muster.md).

## Lizenz

GPL-3.0-or-later ([LICENSE](LICENSE)). Eigener Code; aus Pluma, Xed oder
Mousepad wurde **kein** Code und kein Text übernommen. Logo: offizielles
Querencia-Logo C2 (`data/logo/`, unverändert, Prüfsummen in
`SHA256SUMS.herkunft`).
