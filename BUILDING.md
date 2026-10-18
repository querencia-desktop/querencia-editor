# Bauen

Ubuntu 26.04: `cargo`, `rustc` (≥ 1.92, Ubuntu 1.93.1), `pkgconf`,
`libgtk-4-dev`, `libgtksourceview-5-dev`.

```bash
cargo build --release --locked
```

Paketvorschlag (nicht umgesetzt): `/usr/bin/querencia-editor`,
`/usr/share/applications/org.querencia.Editor.desktop`,
`/usr/share/icons/hicolor/*/apps/org.querencia.Editor.{png,svg}`.

**Symbolgrößen:** bewusst **ohne** `48x48`. Liegt ein Symbol sowohl in
`48x48` als auch in einem Scalable-Verzeichnis (`scalable`, `256x256`), stürzt
GTK 4.22 beim Schließen von Fenstern ab (doppelt freigegebener
`xdg-toplevel-icon`-Puffer, Bericht `querencia-gtk-symbolabsturz-pruefung.md`).
Die 48er-Stufe zeichnet GTK aus dem SVG.
