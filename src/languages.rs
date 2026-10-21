//! Zuordnung Dateiname → GtkSourceView-Sprache.
//!
//! Für die zehn zugesagten Formate steht die Sprache fest (Tabelle), damit
//! z. B. `.conf` nicht je nach Inhalt mal so, mal anders erscheint. Alles
//! andere entscheidet GtkSourceView selbst nach Dateiname und Inhaltstyp.
//! `.txt` ist ausdrücklich reiner Text ohne Hervorhebung.

use crate::i18n::gettext;

/// Feste Zuordnung der Endungen; `Some(None)` heißt „reiner Text“.
pub fn fixed_language_id(file_name: &str) -> Option<Option<&'static str>> {
    let ext = file_name.rsplit_once('.')?.1.to_ascii_lowercase();
    let id = match ext.as_str() {
        "txt" | "text" => None,
        "md" | "markdown" => Some("markdown"),
        "conf" | "ini" | "cfg" => Some("ini"),
        "json" => Some("json"),
        "xml" => Some("xml"),
        "sh" | "bash" => Some("sh"),
        "py" => Some("python3"),
        "rs" => Some("rust"),
        "c" | "h" => Some("c"),
        _ => return None,
    };
    Some(id)
}

/// Sprache für eine Datei: feste Tabelle, sonst Vermutung von GtkSourceView.
pub fn language_for(file_name: &str, content_type: Option<&str>) -> Option<sourceview5::Language> {
    let manager = sourceview5::LanguageManager::default();
    match fixed_language_id(file_name) {
        Some(Some(id)) => manager.language(id),
        Some(None) => None,
        None => manager.guess_language(Some(file_name), content_type),
    }
}

/// Anzeigename für die Statuszeile („Plain Text“ ohne Sprache). Die übrigen
/// Namen übersetzt GtkSourceView selbst (Domäne gtksourceview-5).
pub fn display_name(language: Option<&sourceview5::Language>) -> String {
    match language {
        None => gettext("Plain Text"),
        // GtkSourceView nennt die INI-Sprache „.ini“.
        Some(l) if l.id() == "ini" => gettext("INI/Configuration"),
        Some(l) => l.name().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ten_promised_formats_have_a_fixed_language() {
        let cases = [
            ("notiz.txt", None),
            ("README.md", Some("markdown")),
            ("labwc.conf", Some("ini")),
            ("panel.ini", Some("ini")),
            ("daten.json", Some("json")),
            ("rc.xml", Some("xml")),
            ("start.sh", Some("sh")),
            ("werkzeug.py", Some("python3")),
            ("main.rs", Some("rust")),
            ("hallo.c", Some("c")),
        ];
        for (name, id) in cases {
            assert_eq!(fixed_language_id(name), Some(id), "{name}");
        }
    }

    #[test]
    fn endings_are_case_insensitive_and_unknown_ones_are_guessed() {
        assert_eq!(fixed_language_id("NOTES.MD"), Some(Some("markdown")));
        assert_eq!(fixed_language_id("Makefile"), None);
        assert_eq!(fixed_language_id("bild.png"), None);
        assert_eq!(fixed_language_id(".bashrc"), None);
    }
}

/// Braucht ein Display: läuft nur in der isolierten Sitzung
/// (`cargo test -- --ignored`, siehe docs/testing.md).
#[cfg(test)]
mod gui_tests {
    use super::*;

    #[test]
    #[ignore = "braucht eine (isolierte) Wayland-Sitzung"]
    fn the_ten_formats_resolve_to_gtksourceview_languages() {
        gtk::init().expect("GTK in der isolierten Sitzung");
        sourceview5::init();
        let cases = [
            ("notiz.txt", None),
            ("README.md", Some("markdown")),
            ("labwc.conf", Some("ini")),
            ("panel.ini", Some("ini")),
            ("daten.json", Some("json")),
            ("rc.xml", Some("xml")),
            ("start.sh", Some("sh")),
            ("werkzeug.py", Some("python3")),
            ("main.rs", Some("rust")),
            ("hallo.c", Some("c")),
        ];
        for (name, id) in cases {
            let got = language_for(name, None).map(|l| l.id().to_string());
            assert_eq!(got.as_deref(), id, "{name}");
        }
        for id in ["classic", "classic-dark"] {
            assert!(
                sourceview5::StyleSchemeManager::default()
                    .scheme(id)
                    .is_some(),
                "Farbschema {id}"
            );
        }
    }
}
