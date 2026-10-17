//! Dialoge als gewöhnliche, modale Fenster ohne eigene Titelleiste: labwc
//! dekoriert sie wie jedes andere Fenster (keine GTK-Kopfleiste, kein
//! abgerundeter `GtkAlertDialog`). Escape und der Schließen-Knopf brechen
//! ab; die Antwort wird genau einmal gemeldet.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::APP_NAME;

/// Noch nicht gemeldete Antwort (wird genau einmal genommen).
type PendingAnswer = Rc<RefCell<Option<Box<dyn FnOnce(SaveAnswer)>>>>;

/// Antwort auf „Änderungen speichern?“.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveAnswer {
    Save,
    Discard,
    Cancel,
}

/// Schließt `window` erst im nächsten Leerlauf, nicht aus dem gerade
/// laufenden Tasten- oder Klick-Handler heraus (vorsorglich; der Absturz
/// beim Schließen von Dialogen hatte eine andere Ursache, siehe
/// `set_window_icon`).
pub fn close_later(window: &gtk::Window) {
    let window = window.clone();
    glib::idle_add_local_once(move || window.close());
}

fn modal_window(parent: &impl IsA<gtk::Window>, title: &str) -> gtk::Window {
    let window = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .resizable(false)
        .destroy_with_parent(true)
        .build();
    window.add_css_class("qe-dialog");
    attach_to_app(&window, parent);
    window
}

/// Dialoge gehören zur laufenden Anwendung (wie das Hauptfenster).
///
/// Ohne Anwendung meldet GTK den Programmnamen `querencia-editor` als App-ID
/// und schickt labwc ein eigenes Fenstersymbol (xdg-toplevel-icon). Dabei
/// gibt GTK 4.22 einen der Symbolpuffer sofort und beim Schließen ein zweites
/// Mal frei; der Editor stürzte danach ab (isoliert mit WAYLAND_DEBUG belegt,
/// siehe Bericht). Als Fenster der Anwendung tragen Dialoge die App-ID
/// `org.querencia.Editor`; labwc zeigt das Logo über die Desktop-Datei.
pub fn attach_to_app(window: &gtk::Window, parent: &impl IsA<gtk::Window>) {
    if let Some(app) = parent.as_ref().application() {
        window.set_application(Some(&app));
    }
    set_window_icon(window);
}

/// Name des Fenstersymbols, das GTK labwc schickt (xdg-toplevel-icon).
const WINDOW_ICON: &str = "org.querencia.Editor-fenster";

/// Bettet das Fenstersymbol (offizielles Logo C2, nur SVG) als Ressource ein
/// und macht es dem Symbolthema bekannt; läuft nach dem Start von GTK.
pub fn register_window_icon() {
    let bytes = glib::Bytes::from_static(include_bytes!("../data/resource/editor.gresource"));
    match gio::Resource::from_data(&bytes) {
        Ok(resource) => {
            gio::resources_register(&resource);
            if let Some(display) = gdk::Display::default() {
                gtk::IconTheme::for_display(&display)
                    .add_resource_path("/org/querencia/Editor/icons");
            }
        }
        Err(err) => eprintln!("querencia-editor: Ressourcen nicht lesbar: {err}"),
    }
}

/// Fenstersymbol für labwc setzen – bewusst ein eigener, nur als SVG
/// vorhandener Name statt der App-ID.
///
/// Mit den PNG-Symbolen der App-ID (`hicolor/*/apps/org.querencia.Editor.png`)
/// gibt GTK 4.22 einen der Symbolpuffer gleich nach `set_icon` frei und beim
/// Abbau des Fensters ein zweites Mal; der Editor stürzte danach ab, sobald
/// ein Dialog geschlossen wurde (isoliert mit WAYLAND_DEBUG belegt; mit nur
/// SVG oder ohne Symbol kein Absturz, siehe Bericht). Aus dem SVG erzeugt GTK
/// je Größe eine eigene Textur. labwc zeigt das geschickte Symbol in der
/// Titelleiste, auch ohne eigene SVG-Unterstützung.
pub fn set_window_icon(window: &impl IsA<gtk::Window>) {
    window.as_ref().set_icon_name(Some(WINDOW_ICON));
}

fn close_on_escape(window: &gtk::Window) {
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let w = window.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            if let Some(w) = w.upgrade() {
                close_later(&w);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(keys);
}

fn button_row(buttons: &[&gtk::Button]) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_halign(gtk::Align::End);
    row.set_margin_top(12);
    for b in buttons {
        row.append(*b);
    }
    row
}

/// Frage im Stil von gedit 2.30: Warnsymbol, fette Hauptzeile, Erklärung.
fn message_box(icon: &str, primary: &str, secondary: &str) -> gtk::Box {
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(48);
    image.set_valign(gtk::Align::Start);
    let text = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let p = gtk::Label::new(None);
    p.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(primary)
    ));
    p.set_wrap(true);
    p.set_max_width_chars(48);
    p.set_xalign(0.0);
    let s = gtk::Label::new(Some(secondary));
    s.set_wrap(true);
    s.set_max_width_chars(48);
    s.set_xalign(0.0);
    text.append(&p);
    text.append(&s);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.append(&image);
    row.append(&text);
    row
}

fn content(child: &impl IsA<gtk::Widget>, buttons: &gtk::Box) -> gtk::Box {
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_margin_top(12);
    outer.set_margin_bottom(12);
    outer.set_margin_start(12);
    outer.set_margin_end(12);
    outer.append(child);
    outer.append(buttons);
    outer
}

/// „Änderungen am Dokument „name“ vor dem Schließen speichern?“
pub fn ask_save(
    parent: &impl IsA<gtk::Window>,
    name: &str,
    answer: impl FnOnce(SaveAnswer) + 'static,
) {
    let window = modal_window(parent, APP_NAME);
    let discard = gtk::Button::with_mnemonic("_Ohne Speichern schließen");
    let cancel = gtk::Button::with_mnemonic("_Abbrechen");
    let save = gtk::Button::with_mnemonic("_Speichern");
    save.add_css_class("suggested-action");
    let body = message_box(
        "dialog-warning",
        &format!("Änderungen am Dokument „{name}“ vor dem Schließen speichern?"),
        "Wenn Sie nicht speichern, gehen Ihre Änderungen verloren.",
    );
    window.set_child(Some(&content(
        &body,
        &button_row(&[&discard, &cancel, &save]),
    )));
    window.set_default_widget(Some(&save));
    close_on_escape(&window);

    let answer: PendingAnswer = Rc::new(RefCell::new(Some(Box::new(answer))));
    let reply = {
        let answer = answer.clone();
        let window = window.downgrade();
        move |a: SaveAnswer| {
            let Some(f) = answer.borrow_mut().take() else {
                return; // schon beantwortet (z. B. close-request nach Klick)
            };
            // Erst den Dialog schließen, die Antwort (z. B. Reiter entfernen)
            // im nächsten Leerlauf ausführen – vorsorglich, damit der Abbau
            // des Dialogs und Änderungen am Hauptfenster nicht verschachteln.
            if let Some(w) = window.upgrade() {
                close_later(&w);
            }
            glib::idle_add_local_once(move || f(a));
        }
    };
    let r = reply.clone();
    discard.connect_clicked(move |_| r(SaveAnswer::Discard));
    let r = reply.clone();
    cancel.connect_clicked(move |_| r(SaveAnswer::Cancel));
    let r = reply.clone();
    save.connect_clicked(move |_| r(SaveAnswer::Save));
    // Schließen über labwc oder Escape = Abbrechen.
    window.connect_close_request(move |_| {
        reply(SaveAnswer::Cancel);
        glib::Propagation::Proceed
    });
    window.present();
    save.grab_focus();
}

/// Fehlermeldung mit einem Knopf „Schließen“.
pub fn error(parent: &impl IsA<gtk::Window>, primary: &str, secondary: &str) {
    let window = modal_window(parent, APP_NAME);
    let close = gtk::Button::with_mnemonic("_Schließen");
    let body = message_box("dialog-error", primary, secondary);
    window.set_child(Some(&content(&body, &button_row(&[&close]))));
    window.set_default_widget(Some(&close));
    close_on_escape(&window);
    let w = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    window.present();
}

/// „Gehe zu Zeile“: Zahl eingeben, Eingabetaste springt.
pub fn goto_line(parent: &impl IsA<gtk::Window>, max_line: i32, jump: impl Fn(i32) + 'static) {
    let window = modal_window(parent, "Gehe zu Zeile");
    let label = gtk::Label::with_mnemonic("_Zeile:");
    let spin = gtk::SpinButton::with_range(1.0, f64::from(max_line.max(1)), 1.0);
    label.set_mnemonic_widget(Some(&spin));
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.append(&label);
    row.append(&spin);
    let cancel = gtk::Button::with_mnemonic("_Abbrechen");
    let go = gtk::Button::with_mnemonic("_Springen");
    go.add_css_class("suggested-action");
    window.set_child(Some(&content(&row, &button_row(&[&cancel, &go]))));
    window.set_default_widget(Some(&go));
    close_on_escape(&window);
    let w = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    let w = window.downgrade();
    let s = spin.clone();
    go.connect_clicked(move |_| {
        jump(s.value_as_int());
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    // Eingabetaste im Zahlenfeld springt (activates-default erst ab GTK 4.14).
    let enter = gtk::EventControllerKey::new();
    enter.set_propagation_phase(gtk::PropagationPhase::Capture);
    let g = go.clone();
    enter.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Return || key == gdk::Key::KP_Enter {
            g.emit_clicked();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    spin.add_controller(enter);
    window.present();
    spin.grab_focus();
}

const LOGO_64: &[u8] = include_bytes!("../data/logo/querencia-logo-64.png");

/// „Info“: offizielles Querencia-Logo C2, Name, Version, Lizenz.
pub fn about(parent: &impl IsA<gtk::Window>) {
    let window = modal_window(parent, &format!("Info zu {APP_NAME}"));
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_halign(gtk::Align::Center);
    if let Ok(texture) = gdk::Texture::from_bytes(&glib::Bytes::from_static(LOGO_64)) {
        let image = gtk::Image::from_paintable(Some(&texture));
        image.set_pixel_size(64);
        body.append(&image);
    }
    let name = gtk::Label::new(None);
    name.set_markup(&format!(
        "<b><big>{APP_NAME} {}</big></b>",
        env!("CARGO_PKG_VERSION")
    ));
    let text = gtk::Label::new(Some(
        "Klassischer Texteditor für Querencia Desktop\n\
         nach dem Vorbild von gedit 2.30.\n\n\
         Prototyp – nicht für den Alltag freigegeben.",
    ));
    text.set_justify(gtk::Justification::Center);
    let tech = gtk::Label::new(Some(&format!(
        "GTK {}.{}.{} · GtkSourceView 5",
        gtk::major_version(),
        gtk::minor_version(),
        gtk::micro_version()
    )));
    tech.add_css_class("dim-label");
    let license = gtk::Label::new(Some("Lizenz: GNU GPL, Version 3 oder neuer"));
    license.add_css_class("dim-label");
    body.append(&name);
    body.append(&text);
    body.append(&tech);
    body.append(&license);
    let close = gtk::Button::with_mnemonic("_Schließen");
    window.set_child(Some(&content(&body, &button_row(&[&close]))));
    window.set_default_widget(Some(&close));
    close_on_escape(&window);
    let w = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    window.present();
}
