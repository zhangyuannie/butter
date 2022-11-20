pub mod client;
pub mod config;
pub mod daemon;
pub mod json_file;
pub mod schedule;
pub mod schedule_repo;
pub mod subvolume;

use gtk::prelude::*;
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
