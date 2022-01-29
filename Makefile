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
target/org.zhangyuannie.butter.policy: src/org.zhangyuannie.butter.policy.in

src/config.rs target/org.zhangyuannie.butter.policy:
	sed "s|@LIBEXEC_DIR@|${libexecdir}|g" $< > $@

app: src/config.rs target/org.zhangyuannie.butter.policy
	cargo build --release

install:
	mkdir -p "$(DESTDIR)$(bindir)"
	mkdir -p "$(DESTDIR)$(libexecdir)"
	mkdir -p "$(DESTDIR)$(polkitdir)"

	install -m 0755 target/release/butter "$(DESTDIR)$(bindir)"
	install -m 0755 target/release/butterd "$(DESTDIR)$(libexecdir)"

	install -m 0644 target/org.zhangyuannie.butter.policy "$(DESTDIR)$(polkitdir)"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/butter"
	rm -f "$(DESTDIR)$(libexecdir)/butterd"
	rm -f "$(DESTDIR)$(polkitdir)/org.zhangyuannie.butter.policy"
