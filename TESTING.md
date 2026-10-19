# Testen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                       # ohne Display, mit Katalogtest de.po
python3 werkzeug/i18n.py pruefen          # .pot aktuell, de.po vollständig, Platzhalter, Zugriffstasten
```

Mehrsprachige GUI-Prüfung (G2-AC, je `LANGUAGE=de` mit `LANG=en_US.UTF-8`
wie in der Sitzung und `LANGUAGE=en`, 1024×768 und 1920×1080; Binary und
Katalog aus der Integrationsvorlage, Ablage-Aufbau wie live):

```bash
bash ~/projects/querencia-editor-i18n/werkzeug/lauf.sh de 1024x768   # Bilder und ergebnis.txt nach …/nachweis/gui/de-1024x768/
```

GUI nur isoliert (eigener headless labwc 0.9.8, eigener D-Bus, eigenes
XDG-Profil, virtuelle Eingaben; nie die laufende Sitzung):

```bash
bash ../werkzeug/gui-pruefung.sh          # Bilder und Log nach ../nachweis/gui/
```

Darin läuft auch `cargo test -- --ignored` (GtkSourceView-Sprachen der zehn
Formate, Farbschema).
