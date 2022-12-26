pub use adw::{prelude::*, subclass::prelude::*};
pub use gtk::{prelude::*, subclass::prelude::*};

pub trait BtrWidgetExt {
    fn alert(&self, message: &str);
}

impl<W: IsA<gtk::Widget>> BtrWidgetExt for W {
    fn alert(&self, message: &str) {
        let win = self.root().and_then(|w| w.downcast::<gtk::Window>().ok());
        let dialog = adw::MessageDialog::new(win.as_ref(), None, Some(message));
        dialog.add_response("close", "OK");
        dialog.present();
    }
}
