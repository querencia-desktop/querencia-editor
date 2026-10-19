# Änderungen – Querencia Editor

## [0.2.0] – G2-AC (2026-10-19)

- Mehrsprachigkeit über GNU gettext (`gettext-rs` 0.8 mit dem gettext der
  glibc): alle 75 sichtbaren Texte (Menüs, Dialoge, Statusleiste, Tooltip,
  Meldungen, Desktop-Datei) laufen über die Domäne `querencia-editor`.
- Quellsprache im Code ist Englisch (Wortschatz gedit 2.30); `po/de.po`
  stellt den bisherigen deutschen Wortlaut exakt wieder her. Sprachwahl über
  `LANGUAGE`/`LANG`; ohne Katalog Englisch.
- Kataloge neben der Binary (`share/locale/<lang>/LC_MESSAGES/`), sonst im
  Präfix bzw. `/usr/share/locale`.
- Desktop-Datei aus `data/org.querencia.Editor.desktop.in` mit `[de]`-Zeilen;
  Menüname „Text Editor“/„Texteditor“ und MimeType wie live seit G2-E.
- `werkzeug/i18n.py` (pot, update, mo, desktop, pruefen) ersetzt
  xgettext/msgmerge/msgfmt, solange GNU gettext fehlt; Katalogtest in
  `cargo test`. Muster für die übrigen Programme: `docs/i18n-muster.md`.

## [unveröffentlicht] – G2-V (2026-10-19)

- Eigenes Programmsymbol statt Logo-Platzhalter (QD-034): Notizblock mit
  Ringbindung und Stift in Querencia-Orange; PNG 16–256 und SVG ohne 48x48,
  Fenstersymbol (`org.querencia.Editor-fenster`, GResource) dasselbe SVG.

## [0.1.0] – Prototyp (unveröffentlicht)

- Erster isolierter Prototyp (QE-01): Menüleiste, Reiter, Hervorhebung,
  Suchen/Ersetzen, Zeilennummern, Einrückung, Schutz vor Datenverlust.
