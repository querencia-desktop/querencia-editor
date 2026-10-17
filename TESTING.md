# Testen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                       # ohne Display
```

GUI nur isoliert (eigener headless labwc 0.9.8, eigener D-Bus, eigenes
XDG-Profil, virtuelle Eingaben; nie die laufende Sitzung):

```bash
bash ../werkzeug/gui-pruefung.sh          # Bilder und Log nach ../nachweis/gui/
```

Darin läuft auch `cargo test -- --ignored` (GtkSourceView-Sprachen der zehn
Formate, Farbschema).
