# Änderungen – Querencia Editor

## [0.2.5] – Italienisch, Spanisch, Französisch (2026-10-26)

- Neue Kataloge `po/it.po`, `po/es.po`, `po/fr.po` (je 80 Texte, vollständig);
  Wortwahl nach GNOME/gedit der jeweiligen Sprache und der gemeinsamen
  Querencia-Begriffstabelle, Zugriffstasten je Menü und Dialog eindeutig,
  Online-Hilfe auf querencialinux.org/it|es|fr/docs/.
- Desktop-Datei mit `Name`, `GenericName`, `Comment` und `Keywords` für it,
  es, fr.
- `werkzeug/i18n.py`: Plural-Forms für it und es.

## [0.2.4] – Info-Dialog und Hilfe-Menü (2026-10-22)

- Info-Dialog wie in den übrigen Querencia-Programmen: Hinweis „Alpha –
  nicht für den Alltag freigegeben“ (statt „Prototyp“), Website
  querencialinux.org, „© 2026 Querencia Desktop contributors“, Lizenz
  GPL-3.0-or-later. Englisch heißt er „About Text Editor“ (wie im
  Anwendungsmenü), deutsch weiter „Info zu Querencia Editor“.
- Hilfe-Menü mit „Online-Hilfe“ (querencialinux.org/de/docs/, englische
  Oberfläche /en/docs/) und „Fehler melden“ (neue Fehlermeldung auf GitHub);
  beides öffnet der Standardbrowser der Sitzung.
- Katalog ohne interne Kennungen.

## [0.2.3] – CSS-Welle dunkel, Zugriffstasten (2026-10-21)

- Dunkle Darstellung: Menüleiste und Menüs nehmen ihre Farben ganz aus dem
  Thema Querencia-Klassisch (die gestrichenen festen Werte waren dessen helle
  Werte). Reiterleiste, aktiver Reiter und Statusleiste nehmen die benannten
  Farben des Themas (`@theme_bg_color`, `@theme_base_color`,
  `@theme_fg_color`, `@borders`) statt fester Hellfarben. Hell bleibt alles
  pixelgleich; mit der dunklen Fassung des Themas (0.2) und „Dunkle
  Darstellung bevorzugen“ wird der Editor dunkel, beim Start wie beim
  Wechsel im Lauf. Unter einem Thema ohne dunkle Fassung bleibt er hell.
  Eine Bindung des eigenen CssProviders an `gtk-interface-color-scheme` ist
  damit nicht nötig (isoliert gemessen).
- Textfläche: hell wie bisher das GtkSourceView-Schema „classic“, bei
  dunkler Ansicht des Themas (`@theme_base_color`) dessen eingebaute dunkle
  Fassung „classic-dark“, auch beim Wechsel im Lauf (Entscheidung 21.10.,
  Variante B). Maßgeblich ist das Thema, nicht die Einstellung: unter einem
  Thema ohne dunkle Fassung bleibt auch die Textfläche hell. Ein eigenes
  Schema passend zum Thema ist Folgepunkt für Alpha 2.
- Startfehler GTK 4.22: Nach dem Start meldet der Editor einmal
  `gtk-interface-color-scheme`; sonst bliebe ein unter „dunkel“ gestarteter
  Editor hell.
- Deutsche Zugriffstasten im Such- und Ersetzen-Dialog eindeutig
  (Entscheidung 20.10.): „Schl_ießen“, „_Suchen“,
  „Am Ende _neu beginnen“, dazu „Su_chen nach:“ (sonst S doppelt); „_Ersetzen“
  bleibt. Vorher teilten „S_chließen“/„Su_chen“ das C und „Am _Ende neu
  beginnen“/„_Ersetzen“ das E – Alt+C schloss den Dialog nicht.
- `werkzeug/i18n.py pruefen` meldet doppelte Zugriffstasten je Dialog und
  Menü (`MNEMONIC_GROUPS`, in der msgid und jeder Übersetzung); Tests in
  `werkzeug/test_i18n.py` (mit dem Stand 0.2.2 und „Feld gegen Knopf“ als
  Negativfällen).

## [0.2.2] – 2026-10-20

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

## [0.2.0] – 2026-10-19

- Mehrsprachigkeit über GNU gettext (`gettext-rs` 0.8 mit dem gettext der
  glibc): alle 75 sichtbaren Texte (Menüs, Dialoge, Statusleiste, Tooltip,
  Meldungen, Desktop-Datei) laufen über die Domäne `querencia-editor`.
- Quellsprache im Code ist Englisch (Wortschatz gedit 2.30); `po/de.po`
  stellt den bisherigen deutschen Wortlaut exakt wieder her. Sprachwahl über
  `LANGUAGE`/`LANG`; ohne Katalog Englisch.
- Kataloge neben der Binary (`share/locale/<lang>/LC_MESSAGES/`), sonst im
  Präfix bzw. `/usr/share/locale`.
- Desktop-Datei aus `data/org.querencia.Editor.desktop.in` mit `[de]`-Zeilen;
  Menüname „Text Editor“/„Texteditor“ und MimeType wie bisher.
- `werkzeug/i18n.py` (pot, update, mo, desktop, pruefen) ohne Abhängigkeiten;
  mit GNU gettext 0.23.2 abgeglichen (.mo inhaltsgleich, Desktop-Datei
  byte-gleich zu `msgfmt --desktop`; xgettext liest Rust erst ab 0.24); Katalogtest in
  `cargo test`. Muster für die übrigen Programme: `docs/i18n-muster.md`.

## [unveröffentlicht] – 2026-10-19

- Eigenes Programmsymbol statt Logo-Platzhalter: Notizblock mit
  Ringbindung und Stift in Querencia-Orange; PNG 16–256 und SVG ohne 48x48,
  Fenstersymbol (`org.querencia.Editor-fenster`, GResource) dasselbe SVG.

## [0.1.0] – Prototyp (unveröffentlicht)

- Erster isolierter Prototyp: Menüleiste, Reiter, Hervorhebung,
  Suchen/Ersetzen, Zeilennummern, Einrückung, Schutz vor Datenverlust.
