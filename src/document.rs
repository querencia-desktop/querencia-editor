//! Ein Dokument = ein Reiter: Puffer, Ansicht, Datei und Reiterbeschriftung.
//!
//! Laden und Speichern übernimmt GtkSourceView (`FileLoader`/`FileSaver`):
//! Zeichensatz und Zeilenende werden beim Laden erkannt und beim Speichern
//! beibehalten; neue Dokumente werden als UTF-8 mit LF gespeichert.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};
use sourceview5::prelude::*;

use crate::languages;

pub struct Document {
    pub buffer: sourceview5::Buffer,
    pub view: sourceview5::View,
    pub file: sourceview5::File,
    pub search: sourceview5::SearchContext,
    /// Seite im Notizbuch (enthält die Ansicht).
    pub page: gtk::ScrolledWindow,
    /// Reiterbeschriftung und Schließen-Knopf.
    pub tab: gtk::Box,
    tab_label: gtk::Label,
    pub close_button: gtk::Button,
    /// Nummer für „Unbenanntes Dokument N“.
    untitled: u32,
    /// Wird gerade geladen oder gespeichert (keine zweite Operation).
    pub busy: Cell<bool>,
    title_listeners: RefCell<Vec<Box<dyn Fn()>>>,
}

impl Document {
    pub fn new(untitled: u32, settings: &sourceview5::SearchSettings) -> Rc<Self> {
        let buffer = sourceview5::Buffer::new(None::<&gtk::TextTagTable>);
        buffer.set_highlight_syntax(true);
        buffer.set_highlight_matching_brackets(true);
        if let Some(scheme) = sourceview5::StyleSchemeManager::default().scheme("classic") {
            buffer.set_style_scheme(Some(&scheme));
        }
        let view = sourceview5::View::with_buffer(&buffer);
        // Voreinstellungen wie gedit 2.30: Zeilennummern, automatische
        // Einrückung, Tabulatorbreite 8, Tabulatoren statt Leerzeichen.
        view.set_show_line_numbers(true);
        view.set_auto_indent(true);
        view.set_tab_width(8);
        view.set_insert_spaces_instead_of_tabs(false);
        view.set_highlight_current_line(false);
        view.set_monospace(true);
        view.set_wrap_mode(gtk::WrapMode::None);
        view.set_left_margin(2);

        let page = gtk::ScrolledWindow::builder()
            .child(&view)
            .hexpand(true)
            .vexpand(true)
            .build();
        page.add_css_class("qe-page");

        let tab_label = gtk::Label::new(None);
        let close_button = gtk::Button::from_icon_name("window-close");
        close_button.add_css_class("flat");
        close_button.add_css_class("qe-tab-close");
        close_button.set_tooltip_text(Some("Dokument schließen"));
        close_button.set_focus_on_click(false);
        let tab = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        tab.append(&tab_label);
        tab.append(&close_button);

        let search = sourceview5::SearchContext::new(&buffer, Some(settings));
        search.set_highlight(true);

        let doc = Rc::new(Self {
            buffer,
            view,
            file: sourceview5::File::new(),
            search,
            page,
            tab,
            tab_label,
            close_button,
            untitled,
            busy: Cell::new(false),
            title_listeners: RefCell::new(Vec::new()),
        });
        let weak = Rc::downgrade(&doc);
        doc.buffer.connect_modified_changed(move |_| {
            if let Some(doc) = weak.upgrade() {
                doc.update_title();
            }
        });
        doc.update_title();
        doc
    }

    /// Kurzname: Dateiname oder „Unbenanntes Dokument N“.
    pub fn short_name(&self) -> String {
        self.file
            .location()
            .and_then(|f| f.basename())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| format!("Unbenanntes Dokument {}", self.untitled))
    }

    /// Vollständiger Name für Fenstertitel und Tooltip.
    pub fn full_name(&self) -> String {
        self.file
            .location()
            .map(|f| f.parse_name().to_string())
            .unwrap_or_else(|| self.short_name())
    }

    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }

    /// Unbenannt, nie verändert und leer: wird beim Öffnen einer Datei
    /// ersetzt (wie in gedit).
    pub fn is_pristine(&self) -> bool {
        self.file.location().is_none() && !self.is_modified() && self.buffer.char_count() == 0
    }

    /// Titel ändert sich: Reiter neu beschriften und Zuhörer benachrichtigen.
    pub fn update_title(&self) {
        let star = if self.is_modified() { "*" } else { "" };
        self.tab_label
            .set_text(&format!("{star}{}", self.short_name()));
        self.tab.set_tooltip_text(Some(&self.full_name()));
        for listener in self.title_listeners.borrow().iter() {
            listener();
        }
    }

    pub fn on_title_changed(&self, f: impl Fn() + 'static) {
        self.title_listeners.borrow_mut().push(Box::new(f));
    }

    /// Sprache nach Dateiname (und Inhaltstyp) setzen.
    pub fn update_language(&self) {
        let Some(location) = self.file.location() else {
            self.buffer.set_language(None);
            return;
        };
        let name = location
            .basename()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let content_type = location
            .query_info(
                gio::FILE_ATTRIBUTE_STANDARD_CONTENT_TYPE,
                gio::FileQueryInfoFlags::NONE,
                gio::Cancellable::NONE,
            )
            .ok()
            .and_then(|info| info.content_type());
        let language = languages::language_for(&name, content_type.as_deref());
        self.buffer.set_language(language.as_ref());
    }

    /// Lädt `location` in diesen (leeren) Puffer. `done(Ok)` nach Erfolg.
    pub fn load(
        self: &Rc<Self>,
        location: &gio::File,
        done: impl FnOnce(Result<(), glib::Error>) + 'static,
    ) {
        self.busy.set(true);
        self.file.set_location(Some(location));
        let loader = sourceview5::FileLoader::new(&self.buffer, &self.file);
        let guard = glib::thread_guard::ThreadGuard::new((self.clone(), Box::new(done)));
        loader.load_async(
            glib::Priority::DEFAULT,
            gio::Cancellable::NONE,
            move |result| {
                let (doc, done) = guard.into_inner();
                doc.busy.set(false);
                if result.is_ok() {
                    doc.buffer.set_modified(false);
                    doc.buffer.place_cursor(&doc.buffer.start_iter());
                    doc.update_language();
                }
                doc.update_title();
                done(result);
            },
        );
    }

    /// Speichert unter der aktuellen Datei (muss gesetzt sein).
    pub fn save(self: &Rc<Self>, done: impl FnOnce(Result<(), glib::Error>) + 'static) {
        self.busy.set(true);
        let saver = sourceview5::FileSaver::new(&self.buffer, &self.file);
        // Neue Dokumente: UTF-8, LF. Geladene behalten Zeichensatz und Zeilenende.
        let guard = glib::thread_guard::ThreadGuard::new((self.clone(), Box::new(done)));
        saver.save_async(
            glib::Priority::DEFAULT,
            gio::Cancellable::NONE,
            move |result| {
                let (doc, done) = guard.into_inner();
                doc.busy.set(false);
                if result.is_ok() {
                    doc.buffer.set_modified(false);
                    doc.update_language();
                }
                doc.update_title();
                done(result);
            },
        );
    }

    /// „Zeile Z, Sp. S“ der Einfügemarke (1-basiert, Spalte mit Tabulatorbreite).
    pub fn cursor_position(&self) -> (i32, u32) {
        let iter = self.buffer.iter_at_mark(&self.buffer.get_insert());
        (iter.line() + 1, self.view.visual_column(&iter) + 1)
    }

    /// Einfügemarke auf Zeile `line` (1-basiert) setzen und dorthin rollen.
    pub fn goto_line(&self, line: i32) -> bool {
        let Some(iter) = self.buffer.iter_at_line(line - 1) else {
            return false;
        };
        self.buffer.place_cursor(&iter);
        self.view
            .scroll_to_mark(&self.buffer.get_insert(), 0.0, true, 0.0, 0.3);
        true
    }
}
