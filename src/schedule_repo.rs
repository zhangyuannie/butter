use std::cell::{Ref, RefMut};
use std::path::PathBuf;
use std::sync::MutexGuard;

use anyhow::Result;
use butter::daemon::interface::ButterdClient;
use butter::{json_file::JsonFile, schedule::Schedule};
use glib::once_cell::sync::OnceCell;
use gtk::gio::ListStore;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*};

use crate::client::Client;

mod imp {
    use super::*;

    pub struct ScheduleRepo {
        pub model: ListStore,
        pub client: OnceCell<Client>,
    }

    impl Default for ScheduleRepo {
        fn default() -> Self {
            Self {
                client: Default::default(),
                model: ListStore::new(ScheduleObject::static_type()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScheduleRepo {
        const NAME: &'static str = "ScheduleRepo";
        type Type = super::ScheduleRepo;
    }

    impl ObjectImpl for ScheduleRepo {}
}

glib::wrapper! {
    pub struct ScheduleRepo(ObjectSubclass<imp::ScheduleRepo>);
}

impl ScheduleRepo {
    pub fn new(client: Client) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create ScheduleRepo");
        let imp = ret.imp();
        imp.client.set(client).unwrap();
        ret.sync().unwrap();
        ret
    }
    fn with_client<T, F>(&self, f: F) -> T
    where
        F: FnOnce(MutexGuard<ButterdClient>) -> T,
    {
        f(self.imp().client.get().unwrap().lock().unwrap())
    }

    pub fn model(&self) -> &ListStore {
        &self.imp().model
    }

    pub fn sync(&self) -> Result<()> {
        let mut schedules = self.with_client(|mut c| c.schedules())?;
        schedules.sort_by(|a, b| a.name().cmp(&b.name()));
        let model = self.model();
        model.remove_all();
        for schedule in schedules {
            model.append(&ScheduleObject::new_imp(schedule, true));
        }
        Ok(())
    }

    pub fn persist(&self, obj: &ScheduleObject) -> Result<()> {
        if obj.imp().initial_path.borrow().as_os_str().is_empty() {
            self.with_client(|mut c| c.flush_schedule(obj.borrow().clone()))?;
        } else {
            let mut schedule = obj.borrow().clone();
            let new_path = schedule.path;
            let initial_path = obj.imp().initial_path.borrow().clone();
            schedule.path = initial_path.clone();
            self.with_client(|mut c| c.flush_schedule(schedule))?;
            if initial_path != new_path {
                self.with_client(|mut c| c.fs_rename(initial_path, new_path))?;
            }
        }
        Ok(())
    }

    pub fn delete(&self, obj: &ScheduleObject) -> Result<()> {
        if !obj.imp().initial_path.borrow().as_os_str().is_empty() {
            self.with_client(|mut c| c.fs_remove_file(obj.imp().initial_path.borrow().clone()))?;
        }
        Ok(())
    }
}

mod object_imp {
    use super::*;

    use std::{cell::RefCell, path::PathBuf};

    #[derive(Default)]
    pub struct ScheduleObject {
        pub initial_path: RefCell<PathBuf>,
        pub data: RefCell<JsonFile<Schedule>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScheduleObject {
        const NAME: &'static str = "ScheduleObject";
        type Type = super::ScheduleObject;
    }

    impl ObjectImpl for ScheduleObject {}
}

glib::wrapper! {
    pub struct ScheduleObject(ObjectSubclass<object_imp::ScheduleObject>);
}

impl ScheduleObject {
    fn new_imp(inner: JsonFile<Schedule>, exist: bool) -> Self {
        let obj: Self = glib::Object::new(&[]).expect("Failed to create ScheduleObject");
        let imp = obj.imp();
        if exist {
            *imp.initial_path.borrow_mut() = inner.path.clone();
        }
        *imp.data.borrow_mut() = inner;
        obj
    }

    pub fn new(inner: JsonFile<Schedule>) -> Self {
        Self::new_imp(inner, false)
    }

    pub fn set_path(&self, path: PathBuf) {
        self.imp().data.borrow_mut().path = path;
    }

    pub fn borrow(&self) -> Ref<JsonFile<Schedule>> {
        self.imp().data.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<JsonFile<Schedule>> {
        self.imp().data.borrow_mut()
    }
}

impl Default for ScheduleObject {
    fn default() -> Self {
        Self::new(JsonFile::<Schedule>::default())
    }
}
