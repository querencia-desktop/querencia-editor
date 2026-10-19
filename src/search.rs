//! Suchen und Ersetzen wie in gedit 2.30: eigener, nicht modaler Dialog
//! („Suchen“ bzw. „Ersetzen“) mit den Schaltern Groß-/Kleinschreibung,
//! ganze Wörter, rückwärts und Umbruch am Dokumentende. Gesucht wird mit
//! GtkSourceView (`SearchContext`), alle Treffer werden hervorgehoben.
//! Weitersuchen (Strg+G) und Rückwärts suchen (Umschalt+Strg+G) nutzen
//! denselben Suchbegriff ohne Dialog.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};
use sourceview5::prelude::*;

use crate::document::Document;
use crate::i18n::{gettext, gettext_f, ngettext_f, pgettext};

/// Ergebnis eines Suchschritts (für Statuszeile und Tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Found {
    /// Treffer markiert; `true` = über das Dokumentende hinweg gefunden.
    Yes {
        wrapped: bool,
    },
    No,
    /// Kein Suchbegriff.
    Empty,
}

/// Nächsten (bzw. vorigen) Treffer ab der Auswahl markieren.
pub fn find(doc: &Document, backward: bool) -> Found {
    let settings = doc.search.settings();
    if settings.search_text().is_none_or(|t| t.is_empty()) {
        return Found::Empty;
    }
    let buffer = &doc.buffer;
    let (start, end) = buffer.selection_bounds().unwrap_or_else(|| {
        let i = buffer.iter_at_mark(&buffer.get_insert());
        (i, i)
    });
    let hit = if backward {
        doc.search.backward(&start)
    } else {
        doc.search.forward(&end)
    };
    match hit {
        Some((s, e, wrapped)) => {
            buffer.select_range(&s, &e);
            doc.view
                .scroll_to_mark(&buffer.get_insert(), 0.0, true, 0.0, 0.3);
            Found::Yes { wrapped }
        }
        None => Found::No,
    }
}

/// Ist die aktuelle Auswahl genau ein Treffer, wird sie ersetzt; danach
/// zum nächsten Treffer. Liefert, ob ersetzt wurde.
pub fn replace_one(doc: &Document, replacement: &str) -> Result<bool, glib::Error> {
    let buffer = &doc.buffer;
    let mut replaced = false;
    if let Some((mut s, mut e)) = buffer.selection_bounds()
        && doc.search.occurrence_position(&s, &e) > 0
    {
        doc.search.replace(&mut s, &mut e, replacement)?;
        buffer.place_cursor(&e);
        replaced = true;
    }
    find(doc, false);
    Ok(replaced)
}

/// Alle Treffer ersetzen; liefert die Anzahl.
pub fn replace_all(doc: &Document, replacement: &str) -> Result<u32, glib::Error> {
    // Zählen ohne die asynchrone Zählung von GtkSourceView abzuwarten.
    let mut count = 0u32;
    let mut iter = doc.buffer.start_iter();
    while let Some((_, e, wrapped)) = doc.search.forward(&iter) {
        if wrapped {
            break;
        }
        count += 1;
        iter = e;
    }
    if count > 0 {
        doc.search.replace_all(replacement)?;
    }
    Ok(count)
}

/// Der Such- bzw. Ersetzen-Dialog. `current` liefert das aktive Dokument,
/// `report` schreibt eine Meldung in die Statuszeile.
pub struct SearchDialog {
    window: gtk::Window,
    find_entry: gtk::Entry,
}

impl SearchDialog {
    pub fn new(
        parent: &impl IsA<gtk::Window>,
        settings: &sourceview5::SearchSettings,
        replace: bool,
        current: Rc<dyn Fn() -> Option<Rc<Document>>>,
        report: Rc<dyn Fn(&str)>,
    ) -> Rc<Self> {
        let window = gtk::Window::builder()
            .transient_for(parent)
            .title(if replace {
                gettext("Replace")
            } else {
                gettext("Find")
            })
            .resizable(false)
            .hide_on_close(true)
            .build();
        window.add_css_class("qe-dialog");
        crate::dialogs::attach_to_app(&window, parent);

        let grid = gtk::Grid::builder()
            .row_spacing(6)
            .column_spacing(12)
            .build();
        let find_label = gtk::Label::with_mnemonic(&gettext("_Search for:"));
        find_label.set_xalign(0.0);
        let find_entry = gtk::Entry::builder()
            .hexpand(true)
            .width_chars(30)
            .activates_default(true)
            .build();
        find_label.set_mnemonic_widget(Some(&find_entry));
        grid.attach(&find_label, 0, 0, 1, 1);
        grid.attach(&find_entry, 1, 0, 1, 1);
        let replace_entry = gtk::Entry::builder()
            .hexpand(true)
            .activates_default(true)
            .build();
        if replace {
            let l = gtk::Label::with_mnemonic(&gettext("Replace _with:"));
            l.set_xalign(0.0);
            l.set_mnemonic_widget(Some(&replace_entry));
            grid.attach(&l, 0, 1, 1, 1);
            grid.attach(&replace_entry, 1, 1, 1, 1);
        }
        let case = gtk::CheckButton::with_mnemonic(&gettext("_Match case"));
        let words = gtk::CheckButton::with_mnemonic(&gettext("Match _entire word only"));
        let backward = gtk::CheckButton::with_mnemonic(&gettext("Search _backwards"));
        let wrap = gtk::CheckButton::with_mnemonic(&gettext("Wra_p around"));
        case.set_active(settings.is_case_sensitive());
        words.set_active(settings.is_at_word_boundaries());
        wrap.set_active(settings.wraps_around());
        let checks = gtk::Box::new(gtk::Orientation::Vertical, 2);
        for c in [&case, &words, &backward, &wrap] {
            checks.append(c);
        }
        grid.attach(&checks, 0, 2, 2, 1);

        // TRANSLATORS: Knöpfe im Such-/Ersetzen-Dialog; die Zugriffstasten
        // dürfen sich nicht mit den Feldern darüber überschneiden.
        let close = gtk::Button::with_mnemonic(&pgettext("Search dialog", "_Close"));
        let replace_all_button = gtk::Button::with_mnemonic(&gettext("Replace _All"));
        let replace_button = gtk::Button::with_mnemonic(&gettext("_Replace"));
        let find_button = gtk::Button::with_mnemonic(&gettext("_Find"));
        find_button.add_css_class("suggested-action");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::End);
        row.set_margin_top(12);
        row.append(&close);
        if replace {
            row.append(&replace_all_button);
            row.append(&replace_button);
        }
        row.append(&find_button);
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_margin_top(12);
        outer.set_margin_bottom(12);
        outer.set_margin_start(12);
        outer.set_margin_end(12);
        outer.append(&grid);
        outer.append(&row);
        window.set_child(Some(&outer));
        window.set_default_widget(Some(&find_button));

        // Schalter wirken sofort auf die gemeinsamen Sucheinstellungen.
        let s = settings.clone();
        case.connect_toggled(move |c| s.set_case_sensitive(c.is_active()));
        let s = settings.clone();
        words.connect_toggled(move |c| s.set_at_word_boundaries(c.is_active()));
        let s = settings.clone();
        wrap.connect_toggled(move |c| s.set_wrap_around(c.is_active()));
        let s = settings.clone();
        find_entry.connect_changed(move |e| {
            let text = e.text();
            s.set_search_text(if text.is_empty() {
                None
            } else {
                Some(text.as_str())
            });
        });

        let do_find = {
            let current = current.clone();
            let report = report.clone();
            let backward = backward.clone();
            move || {
                let Some(doc) = current() else { return };
                report(&describe(find(&doc, backward.is_active()), &doc));
            }
        };
        let f = do_find.clone();
        find_button.connect_clicked(move |_| f());
        {
            let current = current.clone();
            let report = report.clone();
            let r = replace_entry.clone();
            replace_button.connect_clicked(move |_| {
                let Some(doc) = current() else { return };
                match replace_one(&doc, &r.text()) {
                    Ok(_) => report(""),
                    Err(e) => report(&replace_failed(&e)),
                }
            });
        }
        {
            let current = current.clone();
            let report = report.clone();
            let r = replace_entry.clone();
            replace_all_button.connect_clicked(move |_| {
                let Some(doc) = current() else { return };
                match replace_all(&doc, &r.text()) {
                    Ok(0) => report(&gettext("Nothing found – nothing replaced.")),
                    Ok(n) => report(&ngettext_f(
                        "Replaced one occurrence.",
                        "Replaced {n} occurrences.",
                        n,
                        &[],
                    )),
                    Err(e) => report(&replace_failed(&e)),
                }
            });
        }
        let w = window.downgrade();
        close.connect_clicked(move |_| {
            if let Some(w) = w.upgrade() {
                crate::dialogs::close_later(&w);
            }
        });
        let keys = gtk::EventControllerKey::new();
        // Vor dem Eingabefeld, das Escape sonst selbst verbraucht.
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let w = window.downgrade();
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(w) = w.upgrade() {
                    crate::dialogs::close_later(&w);
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        window.add_controller(keys);

        Rc::new(Self { window, find_entry })
    }

    /// Fenster endgültig schließen (beim Beenden; sonst wird es nur versteckt).
    pub fn destroy(&self) {
        self.window.destroy();
    }

    /// Zeigen; `initial` (z. B. die Auswahl) wird als Suchbegriff vorbelegt.
    pub fn show(&self, initial: Option<&str>) {
        if let Some(text) = initial.filter(|t| !t.is_empty() && !t.contains('\n')) {
            self.find_entry.set_text(text);
        }
        self.window.present();
        self.find_entry.grab_focus();
    }
}

fn replace_failed(err: &glib::Error) -> String {
    gettext_f("Replace failed: {error}", &[("error", err.message())])
}

/// Meldung für die Statuszeile.
pub fn describe(found: Found, doc: &Document) -> String {
    let text = doc.search.settings().search_text().unwrap_or_default();
    match found {
        Found::Yes { wrapped: true } => {
            gettext("Reached the end of the document, continued from the beginning.")
        }
        Found::Yes { wrapped: false } => String::new(),
        Found::No => gettext_f("“{text}” not found.", &[("text", &text)]),
        Found::Empty => gettext("No search text."),
    }
}
