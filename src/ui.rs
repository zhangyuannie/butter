mod application;
pub mod store;
mod widgets;

use gtk::prelude::*;

pub use application::Application;

pub fn show_error_dialog(parent: Option<&impl IsA<gtk::Window>>, message: &str) {
    let dialog = gtk::MessageDialog::new(
        parent,
        gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Close,
        message,
    );
    dialog.connect_response(|dialog, _| {
        dialog.destroy();
    });
    dialog.show()
}
