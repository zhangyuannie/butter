# Contributing to Butter

## License

By contributing to Butter, you agree that your contributions will be multi-licensed under the MIT License, the [Apache License version 2.0](https://www.apache.org/licenses/LICENSE-2.0) and the [GNU General Public License version 3](https://www.gnu.org/licenses/gpl-3.0.html).

You understand that Zhangyuan Nie can decide which future versions of the GNU General Public License can be used. See section 14 of the GPLv3 for more details.

### What does this mean?

This means that while currently Butter is licensed under GPLv3 only, Zhangyuan Nie can sublicense Butter under the MIT License, the Apache License version 2.0, the GNU General Public License version 3 or any later versions **without your consent**.

### Why?

Some parts of Butter might be separated into their own Rust crates in the future. If that happens, they would need to be dual-licensed under the MIT and Apache 2.0 licenses for maximum compatibility.

## Asking Questions

You can find us on [#org.zhangyuannie.Butter:matrix.org](https://matrix.to/#/#org.zhangyuannie.Butter:matrix.org).

## Pull Requests

- Create your branch from `main` and merge back against it.
- Ideally, you should open an issue first if you want to add a new feature.
- Feel free to open a PR directly for bug fixes, translations and other small changes.

## Libraries

- For system libraries (e.g. gtk), they must be available on the latest supported release of Fedora Linux.
- For rust crates, all versions are fine.

## Translations

1. Join us on [Matrix](https://matrix.to/#/#org.zhangyuannie.Butter:matrix.org) to coordinate and get updated.

2. See [BUILDING.md](BUILDING.md) to set up the project.

3. Use Translation editors like [Poedit](https://poedit.net) to generate the PO file for your language based on the POT file. 
(Note: before submitting the PR, remove the generated .mo files)

4. Add your language to [LINGUAS](po/LINGUAS) if it is not already there.
