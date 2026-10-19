//! Querencia Editor – klassischer Texteditor für Querencia Desktop.
//!
//! Vorbild ist gedit 2.30 (GNOME 2): Menüleiste, Reiter, Statuszeile,
//! Such- und Ersetzen-Dialog. Umgesetzt mit GTK 4 und GtkSourceView 5
//! wie die übrigen Querencia-Programme; kein libadwaita und keine eigene
//! Titelleiste, damit labwc die klassische Server-Titelleiste zeichnet.
//!
//! Eine Instanz je Sitzung (GApplication `org.querencia.Editor`): ein
//! zweiter Aufruf öffnet seine Dateien als Reiter im laufenden Fenster.
//!
//! Oberflächentexte laufen über gettext (Modul `i18n`, Kataloge in `po/`).

mod dialogs;
mod document;
mod editor;
mod i18n;
mod languages;
mod search;

use gtk::prelude::*;
use gtk::{gio, glib};

pub const APP_ID: &str = "org.querencia.Editor";
pub const APP_NAME: &str = "Querencia Editor";

fn main() -> glib::ExitCode {
    i18n::init();
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();
    app.connect_startup(|app| {
        sourceview5::init();
        dialogs::register_window_icon();
        editor::load_css();
        editor::install_accels(app);
    });
    app.connect_activate(|app| {
        let editor = editor::Editor::get(app);
        if editor.is_empty() {
            editor.new_document();
        }
        editor.present();
    });
    app.connect_open(|app, files, _hint| {
        let editor = editor::Editor::get(app);
        for file in files {
            editor.open_file(file);
        }
        editor.present();
    });
    app.run()
}
