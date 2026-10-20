//! Das Editorfenster: Menüleiste, Reiter (GtkNotebook), Statuszeile.
//!
//! Aufbau wie gedit 2.30 ohne Werkzeugleiste und Seitenleiste:
//!   Datei | Bearbeiten | Ansicht | Suchen | Dokumente | Hilfe
//! Keine eigene Titelleiste (`set_titlebar` wird nie gerufen): GTK 4.22
//! fordert dann bei labwc die Server-Dekoration an, labwc zeichnet die
//! klassische Titelleiste mit Titel „*datei – Querencia Editor“.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use sourceview5::prelude::*;

use crate::dialogs::{self, SaveAnswer};
use crate::document::Document;
use crate::i18n::{gettext, gettext_f, pgettext};
use crate::search::{self, SearchDialog};
use crate::{APP_NAME, languages};

const CSS: &str = include_str!("style.css");

/// Rückmeldung nach Speichern/Schließen: `true` = erledigt.
type Done = Box<dyn FnOnce(bool)>;

/// Rückruf einer Fensteraktion.
type ActionFn = Box<dyn Fn(&gio::SimpleAction, Option<&glib::Variant>)>;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Tastenkürzel wie gedit 2.30.
pub fn install_accels(app: &gtk::Application) {
    for (action, keys) in [
        ("win.new", &["<Control>n"][..]),
        ("win.open", &["<Control>o"]),
        ("win.save", &["<Control>s"]),
        ("win.save-as", &["<Shift><Control>s"]),
        ("win.close", &["<Control>w"]),
        ("win.quit", &["<Control>q"]),
        ("win.undo", &["<Control>z"]),
        ("win.redo", &["<Shift><Control>z"]),
        ("win.select-all", &["<Control>a"]),
        ("win.find", &["<Control>f"]),
        ("win.find-next", &["<Control>g"]),
        ("win.find-prev", &["<Shift><Control>g"]),
        ("win.replace", &["<Control>h"]),
        ("win.goto-line", &["<Control>i"]),
        ("win.prev-doc", &["<Control><Alt>Page_Up"]),
        ("win.next-doc", &["<Control><Alt>Page_Down"]),
    ] {
        app.set_accels_for_action(action, keys);
    }
}

fn menu_model() -> gio::Menu {
    fn section(items: &[(String, &str)]) -> gio::Menu {
        let m = gio::Menu::new();
        for (label, action) in items {
            m.append(Some(label), Some(action));
        }
        m
    }
    fn submenu(sections: &[gio::Menu]) -> gio::Menu {
        let m = gio::Menu::new();
        for s in sections {
            m.append_section(None, s);
        }
        m
    }
    // Beschriftungen wie gedit 2.30 (englische msgid, Deutsch aus po/de.po).
    let bar = gio::Menu::new();
    bar.append_submenu(
        Some(&gettext("_File")),
        &submenu(&[
            section(&[
                (gettext("_New"), "win.new"),
                (gettext("_Open…"), "win.open"),
            ]),
            section(&[
                (gettext("_Save"), "win.save"),
                (gettext("Save _As…"), "win.save-as"),
            ]),
            section(&[
                (pgettext("File menu", "_Close"), "win.close"),
                (gettext("_Quit"), "win.quit"),
            ]),
        ]),
    );
    bar.append_submenu(
        Some(&gettext("_Edit")),
        &submenu(&[
            section(&[
                (gettext("_Undo"), "win.undo"),
                (gettext("_Redo"), "win.redo"),
            ]),
            section(&[
                (gettext("Cu_t"), "win.cut"),
                (gettext("_Copy"), "win.copy"),
                (gettext("_Paste"), "win.paste"),
                (gettext("_Delete"), "win.delete"),
            ]),
            section(&[(gettext("Select _All"), "win.select-all")]),
        ]),
    );
    bar.append_submenu(
        Some(&gettext("_View")),
        &submenu(&[section(&[
            (gettext("_Line Numbers"), "win.line-numbers"),
            (gettext("_Text Wrapping"), "win.wrap"),
            (gettext("_Statusbar"), "win.statusbar"),
        ])]),
    );
    bar.append_submenu(
        Some(&gettext("_Search")),
        &submenu(&[
            section(&[
                (gettext("_Find…"), "win.find"),
                (gettext("Find Ne_xt"), "win.find-next"),
                (gettext("Find Pre_vious"), "win.find-prev"),
            ]),
            section(&[(gettext("_Replace…"), "win.replace")]),
            section(&[(gettext("Go to _Line…"), "win.goto-line")]),
        ]),
    );
    bar.append_submenu(
        Some(&gettext("_Documents")),
        &submenu(&[
            section(&[(gettext("_Save All"), "win.save-all")]),
            section(&[
                (gettext("_Previous Document"), "win.prev-doc"),
                (gettext("_Next Document"), "win.next-doc"),
            ]),
        ]),
    );
    bar.append_submenu(
        Some(&gettext("_Help")),
        &submenu(&[section(&[(gettext("_About"), "win.about")])]),
    );
    bar
}

/// „INS“ in der Statusleiste (Einfügemodus).
fn insert_mode_label() -> String {
    // TRANSLATORS: Statusleiste, Einfügemodus (kurz, höchstens 5 Zeichen).
    gettext("INS")
}

pub struct Editor {
    window: gtk::ApplicationWindow,
    notebook: gtk::Notebook,
    docs: RefCell<Vec<Rc<Document>>>,
    status_message: gtk::Label,
    status_language: gtk::Label,
    status_position: gtk::Label,
    status_mode: gtk::Label,
    statusbar: gtk::Box,
    search_settings: sourceview5::SearchSettings,
    find_dialog: RefCell<Option<Rc<SearchDialog>>>,
    replace_dialog: RefCell<Option<Rc<SearchDialog>>>,
    untitled_counter: Cell<u32>,
    line_numbers: Cell<bool>,
    wrap: Cell<bool>,
    /// Fenster wird gerade geschlossen (Fragenkette läuft).
    closing: Cell<bool>,
}

thread_local! {
    static EDITOR: RefCell<Option<Rc<Editor>>> = const { RefCell::new(None) };
}

impl Editor {
    /// Das (einzige) Fenster der Anwendung, bei Bedarf neu angelegt.
    pub fn get(app: &gtk::Application) -> Rc<Self> {
        if let Some(e) = EDITOR.with(|e| e.borrow().clone()) {
            return e;
        }
        let e = Self::build(app);
        EDITOR.with(|slot| *slot.borrow_mut() = Some(e.clone()));
        e
    }

    fn build(app: &gtk::Application) -> Rc<Self> {
        app.set_menubar(Some(&menu_model()));
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(APP_NAME)
            .default_width(760)
            .default_height(560)
            .show_menubar(true)
            .build();
        window.add_css_class("qe-window");
        dialogs::set_window_icon(&window);

        let notebook = gtk::Notebook::builder()
            .scrollable(true)
            .show_border(false)
            .build();
        notebook.set_vexpand(true);

        let status_message = gtk::Label::new(None);
        status_message.set_xalign(0.0);
        status_message.set_hexpand(true);
        status_message.set_ellipsize(gtk::pango::EllipsizeMode::End);
        let status_language = gtk::Label::new(None);
        let status_position = gtk::Label::new(None);
        status_position.set_width_chars(18);
        let status_mode = gtk::Label::new(Some(&insert_mode_label()));
        status_mode.set_width_chars(5);
        let statusbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        statusbar.add_css_class("qe-statusbar");
        for w in [
            &status_message,
            &status_language,
            &status_position,
            &status_mode,
        ] {
            statusbar.append(w);
        }

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.append(&notebook);
        vbox.append(&statusbar);
        window.set_child(Some(&vbox));

        let search_settings = sourceview5::SearchSettings::new();
        search_settings.set_wrap_around(true);

        let editor = Rc::new(Self {
            window,
            notebook,
            docs: RefCell::new(Vec::new()),
            status_message,
            status_language,
            status_position,
            status_mode,
            statusbar,
            search_settings,
            find_dialog: RefCell::new(None),
            replace_dialog: RefCell::new(None),
            untitled_counter: Cell::new(0),
            line_numbers: Cell::new(true),
            wrap: Cell::new(false),
            closing: Cell::new(false),
        });
        editor.install_actions();

        let weak = Rc::downgrade(&editor);
        editor.notebook.connect_switch_page(move |_, page, _| {
            if let Some(e) = weak.upgrade() {
                // Seite wird erst danach aktuell: Dokument über die Seite suchen.
                let doc = e
                    .docs
                    .borrow()
                    .iter()
                    .find(|d| d.page.upcast_ref::<gtk::Widget>() == page)
                    .cloned();
                if let Some(doc) = doc {
                    e.refresh(&doc);
                    doc.view.grab_focus();
                }
            }
        });
        let weak = Rc::downgrade(&editor);
        editor.window.connect_close_request(move |_| {
            if let Some(e) = weak.upgrade() {
                e.close_all_then_quit();
            }
            glib::Propagation::Stop
        });
        // 0.2.3: Textflächen folgen einem Wechsel des Farbschemas im Lauf.
        if let Some(settings) = gtk::Settings::default() {
            let weak = Rc::downgrade(&editor);
            settings.connect_notify_local(Some("gtk-interface-color-scheme"), move |_, _| {
                if let Some(e) = weak.upgrade() {
                    for d in e.docs.borrow().iter() {
                        d.follow_color_scheme();
                    }
                }
            });
        }
        editor
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn is_empty(&self) -> bool {
        self.docs.borrow().is_empty()
    }

    fn current(&self) -> Option<Rc<Document>> {
        let page = self.notebook.nth_page(self.notebook.current_page())?;
        self.docs
            .borrow()
            .iter()
            .find(|d| d.page.upcast_ref::<gtk::Widget>() == &page)
            .cloned()
    }

    fn report(&self, text: &str) {
        self.status_message.set_text(text);
    }

    /// Fenstertitel und Statuszeile für das aktive Dokument.
    fn refresh(&self, doc: &Document) {
        let star = if doc.is_modified() { "*" } else { "" };
        self.window
            .set_title(Some(&format!("{star}{} – {APP_NAME}", doc.short_name())));
        self.status_language
            .set_text(&languages::display_name(doc.buffer.language().as_ref()));
        let (line, col) = doc.cursor_position();
        self.status_position.set_text(&gettext_f(
            // TRANSLATORS: Statusleiste, Position der Einfügemarke (wie gedit „Ln 12, Col 5“).
            "Ln {line}, Col {col}",
            &[("line", &line.to_string()), ("col", &col.to_string())],
        ));
        self.status_mode.set_text(&if doc.view.overwrites() {
            // TRANSLATORS: Statusleiste, Überschreibmodus (kurz, höchstens 5 Zeichen).
            gettext("OVR")
        } else {
            insert_mode_label()
        });
    }

    fn is_current(&self, doc: &Document) -> bool {
        self.current().is_some_and(|c| std::ptr::eq(&*c, doc))
    }

    /// Neuer Reiter. Nur unbenannte Dokumente zählen für
    /// „Unbenanntes Dokument N“ (wie gedit).
    fn add_document(self: &Rc<Self>, untitled: bool) -> Rc<Document> {
        let n = if untitled {
            self.untitled_counter.set(self.untitled_counter.get() + 1);
            self.untitled_counter.get()
        } else {
            0
        };
        let doc = Document::new(n, &self.search_settings);
        doc.view.set_show_line_numbers(self.line_numbers.get());
        doc.view.set_wrap_mode(if self.wrap.get() {
            gtk::WrapMode::WordChar
        } else {
            gtk::WrapMode::None
        });

        let page_num = self.notebook.append_page(&doc.page, Some(&doc.tab));
        self.notebook.set_tab_reorderable(&doc.page, true);
        self.docs.borrow_mut().push(doc.clone());

        let weak_e = Rc::downgrade(self);
        let weak_d = Rc::downgrade(&doc);
        doc.on_title_changed(move || {
            if let (Some(e), Some(d)) = (weak_e.upgrade(), weak_d.upgrade())
                && e.is_current(&d)
            {
                e.refresh(&d);
            }
        });
        let weak_e = Rc::downgrade(self);
        let weak_d = Rc::downgrade(&doc);
        doc.buffer.connect_cursor_position_notify(move |_| {
            if let (Some(e), Some(d)) = (weak_e.upgrade(), weak_d.upgrade())
                && e.is_current(&d)
            {
                e.refresh(&d);
            }
        });
        let weak_e = Rc::downgrade(self);
        let weak_d = Rc::downgrade(&doc);
        doc.view.connect_overwrite_notify(move |_| {
            if let (Some(e), Some(d)) = (weak_e.upgrade(), weak_d.upgrade())
                && e.is_current(&d)
            {
                e.refresh(&d);
            }
        });
        let weak_e = Rc::downgrade(self);
        let weak_d = Rc::downgrade(&doc);
        doc.close_button.connect_clicked(move |_| {
            if let (Some(e), Some(d)) = (weak_e.upgrade(), weak_d.upgrade()) {
                e.close_document(&d, |_| {});
            }
        });

        self.notebook.set_current_page(Some(page_num));
        self.notebook.set_show_tabs(true);
        self.refresh(&doc);
        doc.view.grab_focus();
        doc
    }

    pub fn new_document(self: &Rc<Self>) {
        self.add_document(true);
    }

    /// Öffnet `file` in einem neuen Reiter; ist das aktive Dokument leer und
    /// unbenannt, wird es ersetzt. Bereits offene Dateien werden nur gezeigt.
    pub fn open_file(self: &Rc<Self>, file: &gio::File) {
        let existing = self
            .docs
            .borrow()
            .iter()
            .find(|d| d.file.location().is_some_and(|l| l.equal(file)))
            .cloned();
        if let Some(doc) = existing {
            if let Some(n) = self.notebook.page_num(&doc.page) {
                self.notebook.set_current_page(Some(n));
            }
            return;
        }
        let reuse = self.current().filter(|d| d.is_pristine() && !d.busy.get());
        let doc = reuse.unwrap_or_else(|| self.add_document(false));
        let weak = Rc::downgrade(self);
        let name = file.parse_name().to_string();
        let d2 = doc.clone();
        doc.load(file, move |result| {
            let Some(e) = weak.upgrade() else { return };
            match result {
                Ok(()) => {
                    e.report("");
                    if e.is_current(&d2) {
                        e.refresh(&d2);
                    }
                }
                Err(err) => {
                    // Reiter wieder entfernen, Datei unverändert lassen.
                    e.remove_document(&d2);
                    dialogs::error(
                        &e.window,
                        &gettext_f("Could not open “{name}”.", &[("name", &name)]),
                        err.message(),
                    );
                }
            }
        });
    }

    fn remove_document(self: &Rc<Self>, doc: &Rc<Document>) {
        if let Some(n) = self.notebook.page_num(&doc.page) {
            self.notebook.remove_page(Some(n));
        }
        self.docs.borrow_mut().retain(|d| !Rc::ptr_eq(d, doc));
        if let Some(cur) = self.current() {
            self.refresh(&cur);
        } else {
            self.window.set_title(Some(APP_NAME));
            self.status_language.set_text("");
            self.status_position.set_text("");
        }
    }

    /// Speichern; ohne Datei über „Speichern unter“. `done(true)` = gespeichert.
    fn save(self: &Rc<Self>, doc: &Rc<Document>, done: Done) {
        if doc.busy.get() {
            done(false);
            return;
        }
        if doc.file.location().is_none() {
            self.save_as(doc, done);
            return;
        }
        let weak = Rc::downgrade(self);
        let name = doc.short_name();
        let full = doc.full_name();
        doc.save(move |result| {
            let Some(e) = weak.upgrade() else { return };
            match result {
                Ok(()) => {
                    e.report(&gettext_f("Saved “{name}”.", &[("name", &name)]));
                    done(true);
                }
                Err(err) => {
                    dialogs::error(
                        &e.window,
                        &gettext_f("Could not save “{name}”.", &[("name", &full)]),
                        err.message(),
                    );
                    done(false);
                }
            }
        });
    }

    fn save_as(self: &Rc<Self>, doc: &Rc<Document>, done: Done) {
        let dialog = gtk::FileDialog::builder()
            .title(gettext("Save As…"))
            .modal(true)
            .initial_name(doc.short_name())
            .build();
        if let Some(parent) = doc.file.location().and_then(|f| f.parent()) {
            dialog.set_initial_folder(Some(&parent));
        }
        let weak = Rc::downgrade(self);
        let doc = doc.clone();
        dialog.save(Some(&self.window), gio::Cancellable::NONE, move |result| {
            let Some(e) = weak.upgrade() else { return };
            match result {
                Ok(target) => {
                    let previous = doc.file.location();
                    doc.file.set_location(Some(&target));
                    let doc2 = doc.clone();
                    e.save(
                        &doc,
                        Box::new(move |ok| {
                            if !ok {
                                doc2.file.set_location(previous.as_ref());
                            }
                            doc2.update_title();
                            done(ok);
                        }),
                    );
                }
                Err(_) => done(false), // abgebrochen
            }
        });
    }

    /// Reiter schließen; bei Änderungen vorher fragen. `done(true)` = geschlossen.
    fn close_document(self: &Rc<Self>, doc: &Rc<Document>, done: impl FnOnce(bool) + 'static) {
        if !doc.is_modified() {
            self.remove_document(doc);
            done(true);
            return;
        }
        if let Some(n) = self.notebook.page_num(&doc.page) {
            self.notebook.set_current_page(Some(n));
        }
        let weak = Rc::downgrade(self);
        let doc = doc.clone();
        dialogs::ask_save(&self.window, &doc.short_name(), move |answer| {
            let Some(e) = weak.upgrade() else { return };
            match answer {
                SaveAnswer::Discard => {
                    e.remove_document(&doc);
                    done(true);
                }
                SaveAnswer::Cancel => done(false),
                SaveAnswer::Save => {
                    let e2 = e.clone();
                    let d2 = doc.clone();
                    e.save(
                        &doc,
                        Box::new(move |ok| {
                            if ok {
                                e2.remove_document(&d2);
                            }
                            done(ok);
                        }),
                    );
                }
            }
        });
    }

    /// Alle Reiter nacheinander schließen (mit Rückfragen), dann beenden.
    /// Ein „Abbrechen“ hält an; das Fenster bleibt offen.
    fn close_all_then_quit(self: &Rc<Self>) {
        if self.closing.replace(true) {
            return;
        }
        // Wie gedit: erst für jedes geänderte Dokument fragen, dann beenden.
        // Bis dahin bleibt jeder Reiter offen; „Abbrechen“ ändert nichts.
        fn next(e: Rc<Editor>, answered: Rc<RefCell<Vec<Rc<Document>>>>) {
            let pending = e
                .docs
                .borrow()
                .iter()
                .find(|d| d.is_modified() && !answered.borrow().iter().any(|a| Rc::ptr_eq(a, d)))
                .cloned();
            let Some(doc) = pending else {
                // Offene Such-Dialoge gehören zur Anwendung und hielten sie
                // sonst am Leben.
                for slot in [&e.find_dialog, &e.replace_dialog] {
                    if let Some(d) = slot.borrow_mut().take() {
                        d.destroy();
                    }
                }
                e.window.destroy();
                EDITOR.with(|slot| *slot.borrow_mut() = None);
                return;
            };
            if let Some(n) = e.notebook.page_num(&doc.page) {
                e.notebook.set_current_page(Some(n));
            }
            let e2 = e.clone();
            let d2 = doc.clone();
            dialogs::ask_save(&e.window, &doc.short_name(), move |answer| match answer {
                SaveAnswer::Cancel => e2.closing.set(false),
                SaveAnswer::Discard => {
                    answered.borrow_mut().push(d2);
                    next(e2, answered);
                }
                SaveAnswer::Save => {
                    let e3 = e2.clone();
                    e2.save(
                        &d2,
                        Box::new(move |ok| {
                            if ok {
                                next(e3, answered);
                            } else {
                                e3.closing.set(false);
                            }
                        }),
                    );
                }
            });
        }
        next(self.clone(), Rc::new(RefCell::new(Vec::new())));
    }

    fn with_current(
        self: &Rc<Self>,
        f: impl Fn(&Rc<Self>, &Rc<Document>) + 'static,
    ) -> impl Fn(&gio::SimpleAction, Option<&glib::Variant>) + 'static {
        let weak = Rc::downgrade(self);
        move |_, _| {
            if let Some(e) = weak.upgrade()
                && let Some(doc) = e.current()
            {
                f(&e, &doc);
            }
        }
    }

    fn search_dialog(self: &Rc<Self>, replace: bool) -> Rc<SearchDialog> {
        let slot = if replace {
            &self.replace_dialog
        } else {
            &self.find_dialog
        };
        if let Some(d) = slot.borrow().clone() {
            return d;
        }
        let weak = Rc::downgrade(self);
        let current: Rc<dyn Fn() -> Option<Rc<Document>>> =
            Rc::new(move || weak.upgrade().and_then(|e| e.current()));
        let weak = Rc::downgrade(self);
        let report: Rc<dyn Fn(&str)> = Rc::new(move |t| {
            if let Some(e) = weak.upgrade() {
                e.report(t);
            }
        });
        let d = SearchDialog::new(
            &self.window,
            &self.search_settings,
            replace,
            current,
            report,
        );
        *slot.borrow_mut() = Some(d.clone());
        d
    }

    fn install_actions(self: &Rc<Self>) {
        let add = |name: &str, f: ActionFn| {
            let a = gio::SimpleAction::new(name, None);
            a.connect_activate(f);
            self.window.add_action(&a);
        };
        let weak = Rc::downgrade(self);
        add(
            "new",
            Box::new(move |_, _| {
                if let Some(e) = weak.upgrade() {
                    e.new_document();
                }
            }),
        );
        let weak = Rc::downgrade(self);
        add(
            "open",
            Box::new(move |_, _| {
                let Some(e) = weak.upgrade() else { return };
                let dialog = gtk::FileDialog::builder()
                    .title(gettext("Open File"))
                    .modal(true)
                    .build();
                if let Some(dir) = e
                    .current()
                    .and_then(|d| d.file.location())
                    .and_then(|f| f.parent())
                {
                    dialog.set_initial_folder(Some(&dir));
                }
                let weak = Rc::downgrade(&e);
                dialog.open_multiple(Some(&e.window), gio::Cancellable::NONE, move |result| {
                    let (Some(e), Ok(files)) = (weak.upgrade(), result) else {
                        return;
                    };
                    for i in 0..files.n_items() {
                        if let Some(f) = files.item(i).and_downcast::<gio::File>() {
                            e.open_file(&f);
                        }
                    }
                });
            }),
        );
        add(
            "save",
            Box::new(self.with_current(|e, d| e.save(d, Box::new(|_| {})))),
        );
        add(
            "save-as",
            Box::new(self.with_current(|e, d| e.save_as(d, Box::new(|_| {})))),
        );
        let weak = Rc::downgrade(self);
        add(
            "save-all",
            Box::new(move |_, _| {
                let Some(e) = weak.upgrade() else { return };
                let docs: Vec<_> = e
                    .docs
                    .borrow()
                    .iter()
                    .filter(|d| d.is_modified())
                    .cloned()
                    .collect();
                for d in docs {
                    e.save(&d, Box::new(|_| {}));
                }
            }),
        );
        add(
            "close",
            Box::new(self.with_current(|e, d| e.close_document(d, |_| {}))),
        );
        let weak = Rc::downgrade(self);
        add(
            "quit",
            Box::new(move |_, _| {
                if let Some(e) = weak.upgrade() {
                    e.close_all_then_quit();
                }
            }),
        );
        add(
            "undo",
            Box::new(self.with_current(|_, d| {
                if d.buffer.can_undo() {
                    d.buffer.undo();
                }
            })),
        );
        add(
            "redo",
            Box::new(self.with_current(|_, d| {
                if d.buffer.can_redo() {
                    d.buffer.redo();
                }
            })),
        );
        add(
            "cut",
            Box::new(self.with_current(|_, d| d.buffer.cut_clipboard(&d.view.clipboard(), true))),
        );
        add(
            "copy",
            Box::new(self.with_current(|_, d| d.buffer.copy_clipboard(&d.view.clipboard()))),
        );
        add(
            "paste",
            Box::new(
                self.with_current(|_, d| d.buffer.paste_clipboard(&d.view.clipboard(), None, true)),
            ),
        );
        add(
            "delete",
            Box::new(self.with_current(|_, d| {
                d.buffer.delete_selection(true, true);
            })),
        );
        add(
            "select-all",
            Box::new(self.with_current(|_, d| {
                let (s, e) = d.buffer.bounds();
                d.buffer.select_range(&s, &e);
            })),
        );

        // Umschalter (Häkchen im Menü), gelten für alle Reiter.
        for (name, initial) in [("line-numbers", true), ("wrap", false), ("statusbar", true)] {
            let a = gio::SimpleAction::new_stateful(name, None, &initial.to_variant());
            let weak = Rc::downgrade(self);
            let n = name.to_string();
            a.connect_activate(move |a, _| {
                let Some(e) = weak.upgrade() else { return };
                let on = !a.state().and_then(|s| s.get::<bool>()).unwrap_or(false);
                a.set_state(&on.to_variant());
                match n.as_str() {
                    "line-numbers" => {
                        e.line_numbers.set(on);
                        for d in e.docs.borrow().iter() {
                            d.view.set_show_line_numbers(on);
                        }
                    }
                    "wrap" => {
                        e.wrap.set(on);
                        let mode = if on {
                            gtk::WrapMode::WordChar
                        } else {
                            gtk::WrapMode::None
                        };
                        for d in e.docs.borrow().iter() {
                            d.view.set_wrap_mode(mode);
                        }
                    }
                    _ => e.statusbar.set_visible(on),
                }
            });
            self.window.add_action(&a);
        }

        for replace in [false, true] {
            let weak = Rc::downgrade(self);
            add(
                if replace { "replace" } else { "find" },
                Box::new(move |_, _| {
                    let Some(e) = weak.upgrade() else { return };
                    let selected = e.current().and_then(|d| {
                        d.buffer
                            .selection_bounds()
                            .map(|(s, en)| d.buffer.text(&s, &en, false).to_string())
                    });
                    e.search_dialog(replace).show(selected.as_deref());
                }),
            );
        }
        for backward in [false, true] {
            add(
                if backward { "find-prev" } else { "find-next" },
                Box::new(self.with_current(move |e, d| {
                    let found = search::find(d, backward);
                    e.report(&search::describe(found, d));
                })),
            );
        }
        add(
            "goto-line",
            Box::new(self.with_current(|e, d| {
                let d2 = d.clone();
                dialogs::goto_line(&e.window, d.buffer.line_count(), move |line| {
                    d2.goto_line(line);
                    d2.view.grab_focus();
                });
            })),
        );
        for (name, step) in [("prev-doc", -1i32), ("next-doc", 1)] {
            let weak = Rc::downgrade(self);
            add(
                name,
                Box::new(move |_, _| {
                    let Some(e) = weak.upgrade() else { return };
                    let n = e.notebook.n_pages() as i32;
                    if n > 0 {
                        let cur = e.notebook.current_page().map(|p| p as i32).unwrap_or(0);
                        e.notebook
                            .set_current_page(Some(((cur + step).rem_euclid(n)) as u32));
                    }
                }),
            );
        }
        let weak = Rc::downgrade(self);
        add(
            "about",
            Box::new(move |_, _| {
                if let Some(e) = weak.upgrade() {
                    dialogs::about(&e.window);
                }
            }),
        );
    }
}
