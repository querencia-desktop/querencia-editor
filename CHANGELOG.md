# Änderungen – Querencia Editor

## [0.2.2] – QD-076 (2026-10-20)

- Das Fenster „Suchen“ bzw. „Ersetzen“ (Strg+F, Strg+H) wird bei jedem
  Öffnen neu gebaut und beim Schließen zerstört, statt verborgen und wieder
  gezeigt zu werden: labwc 0.9.8 verknüpft die Serverdekoration nur mit dem
  ersten xdg_toplevel einer Fläche, GTK 4.22 legt beim Wiederzeigen ein
  neues an – ab dem zweiten Öffnen fehlte die Titelleiste. Muster Bilder
  0.2.3 (`WeakRef`, `destroy_with_parent`, Schließen im Leerlauf).
- Suchbegriff, Ersetzungstext und „Rückwärts suchen“ bleiben über das
  Schließen hinaus erhalten wie bisher; die übrigen Schalter kommen wie
  bisher aus den gemeinsamen Sucheinstellungen. Texte und Katalog
  unverändert.

## [0.2.1] – CSS-Welle (2026-10-20)

- Auswahlschrift dunkel (`#1a1a1a` statt `#ffffff`) auf dem orangen Verlauf
  der Menüleiste und der Menüeinträge, passend zum GTK-Thema
  Querencia-Klassisch (Entscheidung 20.10.: dunkle Auswahlschrift). Kontrast
  6,2:1 (unten, `#f07746`) bis 8,5:1 (oben, `#f6a179`), mit Weiß waren es
  2,8:1 bzw. 2,0:1.
  Verlauf und alles Übrige unverändert; Texte und Katalog unverändert.

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
- `werkzeug/i18n.py` (pot, update, mo, desktop, pruefen) ohne Abhängigkeiten;
  mit GNU gettext 0.23.2 abgeglichen (.mo inhaltsgleich, Desktop-Datei
  byte-gleich zu `msgfmt --desktop`; xgettext liest Rust erst ab 0.24); Katalogtest in
  `cargo test`. Muster für die übrigen Programme: `docs/i18n-muster.md`.

## [unveröffentlicht] – G2-V (2026-10-19)

- Eigenes Programmsymbol statt Logo-Platzhalter (QD-034): Notizblock mit
  Ringbindung und Stift in Querencia-Orange; PNG 16–256 und SVG ohne 48x48,
  Fenstersymbol (`org.querencia.Editor-fenster`, GResource) dasselbe SVG.

## [0.1.0] – Prototyp (unveröffentlicht)

- Erster isolierter Prototyp (QE-01): Menüleiste, Reiter, Hervorhebung,
  Suchen/Ersetzen, Zeilennummern, Einrückung, Schutz vor Datenverlust.
