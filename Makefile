SHELL = /bin/sh

prefix = /usr
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libexecdir = $(exec_prefix)/libexec
datarootdir = $(prefix)/share
datadir =  $(datarootdir)

polkitdir = $(datadir)/polkit-1/actions

all: app

src/config.rs: src/config.rs.in
target/org.zhangyuannie.butter.policy: data/org.zhangyuannie.butter.policy.in

src/config.rs target/org.zhangyuannie.butter.policy:
	sed "s|@LIBEXEC_DIR@|$(libexecdir)|g" $< > $@

app: src/config.rs target/org.zhangyuannie.butter.policy
	cargo build --release

install:
	mkdir -p "$(DESTDIR)$(bindir)"
	mkdir -p "$(DESTDIR)$(libexecdir)"
	mkdir -p "$(DESTDIR)$(polkitdir)"
	mkdir -p "$(DESTDIR)$(datadir)/icons/hicolor/scalable/apps"
	mkdir -p "$(DESTDIR)$(datadir)/icons/hicolor/symbolic/apps"
	mkdir -p "$(DESTDIR)$(datadir)/applications"

	install -m 0755 target/release/butter "$(DESTDIR)$(bindir)"
	install -Tm 0755 src/daemon/main.py "$(DESTDIR)$(libexecdir)/butterd"

	install -m 0644 target/org.zhangyuannie.butter.policy "$(DESTDIR)$(polkitdir)"
	install -m 0644 data/org.zhangyuannie.butter.desktop "$(DESTDIR)$(datadir)/applications"
	install -m 0644 data/icons/hicolor/scalable/apps/org.zhangyuannie.butter.svg "$(DESTDIR)$(datadir)/icons/hicolor/scalable/apps"
	install -m 0644 data/icons/hicolor/symbolic/apps/org.zhangyuannie.butter-symbolic.svg "$(DESTDIR)$(datadir)/icons/hicolor/symbolic/apps"

	gtk-update-icon-cache -qtf "$(DESTDIR)$(datadir)/icons/hicolor"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/butter"
	rm -f "$(DESTDIR)$(libexecdir)/butterd"
	rm -f "$(DESTDIR)$(polkitdir)/org.zhangyuannie.butter.policy"
	rm -f "$(DESTDIR)$(datadir)/applications/org.zhangyuannie.butter.desktop"
	rm -f "$(DESTDIR)$(datadir)/icons/hicolor/scalable/apps/org.zhangyuannie.butter.svg"
	rm -f "$(DESTDIR)$(datadir)/icons/hicolor/symbolic/apps/org.zhangyuannie.butter-symbolic.svg"
