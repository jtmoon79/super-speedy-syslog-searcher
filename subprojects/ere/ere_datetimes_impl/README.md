# `ere_datetimes_impl`

This crate is only for project [super-speedy-syslog-searcher]. It has no use outside of it.

[super-speedy-syslog-searcher]: https://github.com/jtmoon79/super-speedy-syslog-searcher/

This crate defines [`ere`] regular expressions used by **super-speedy-syslog-searcher**. It requires tens of minutes to compile.
Moving these regular expressions into a separate crate allows avoiding recompilation when other project code is changed.

[`ere`]: https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/main/subprojects/ere
