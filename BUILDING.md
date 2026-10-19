# Bauen

Ubuntu 26.04: `cargo`, `rustc` (≥ 1.92, Ubuntu 1.93.1), `pkgconf`,
`libgtk-4-dev`, `libgtksourceview-5-dev`.

```bash
cargo build --release --locked
```

## Übersetzungen

Texte im Code sind englisch (msgid), Deutsch steht in `po/de.po`. Kataloge
erzeugen (ohne GNU gettext, reines Python):

```bash
python3 werkzeug/i18n.py pruefen               # .pot aktuell, de.po vollständig
python3 werkzeug/i18n.py mo <prefix>/share/locale
```

Die Binary sucht `<lang>/LC_MESSAGES/querencia-editor.mo` zuerst in
`<ordner der binary>/share/locale`, dann in `<ordner>/../share/locale`,
zuletzt in `/usr/share/locale`; `QUERENCIA_EDITOR_LOCALEDIR` setzt das
Verzeichnis ausdrücklich. Nach Textänderungen: `i18n.py pot`, `update`,
`de.po` ergänzen, `desktop data/org.querencia.Editor.desktop.in
data/org.querencia.Editor.desktop`. Einzelheiten: `docs/i18n-muster.md`.

Paketvorschlag (nicht umgesetzt): `/usr/bin/querencia-editor`,
`/usr/share/locale/de/LC_MESSAGES/querencia-editor.mo` (aus `po/de.po`,
Build-Abhängigkeit `gettext` für `msgfmt -c`),
`/usr/share/applications/org.querencia.Editor.desktop`,
`/usr/share/icons/hicolor/*/apps/org.querencia.Editor.{png,svg}`.

**Symbolgrößen:** bewusst **ohne** `48x48`. Liegt ein Symbol sowohl in
`48x48` als auch in einem Scalable-Verzeichnis (`scalable`, `256x256`), stürzt
GTK 4.22 beim Schließen von Fenstern ab (doppelt freigegebener
`xdg-toplevel-icon`-Puffer, Bericht `querencia-gtk-symbolabsturz-pruefung.md`).
Die 48er-Stufe zeichnet GTK aus dem SVG.
