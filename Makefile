SHELL = /bin/sh

prefix = /usr
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libexecdir = $(exec_prefix)/libexec
datarootdir = $(prefix)/share
datadir =  $(datarootdir)

polkitdir = $(datadir)/polkit-1/actions

all: app

app:
	cargo build --release

install:
	mkdir -p "$(DESTDIR)$(bindir)"
	mkdir -p "$(DESTDIR)$(libexecdir)"
	mkdir -p "$(DESTDIR)$(polkitdir)"

	install -m 0755 -T data/launch.sh "$(DESTDIR)$(bindir)/butter"
	install -m 0755 target/release/butter "$(DESTDIR)$(libexecdir)"

	install -m 0644 data/org.zhangyuannie.butter.policy "$(DESTDIR)$(polkitdir)"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/butter"
	rm -f "$(DESTDIR)$(libexecdir)/butter"
	rm -f "$(DESTDIR)$(polkitdir)/org.zhangyuannie.butter.policy"
