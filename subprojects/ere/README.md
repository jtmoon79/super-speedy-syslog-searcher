# `ere` - Extended Regular Expressions

The code under paths `ere`, `ere-core`, and `ere-macros` is copied from [`ere` Pull Request #4](https://github.com/2kai2kai2/ere/pull/4).
It is [licensed under the MIT License](https://github.com/2kai2kai2/ere/blob/9ae714909f24e025612e385419af17aaed843a60/LICENSE).
It has been modified for this project, *Super Speedy Syslog Searcher*.

Code under paths `ere_automator_procmacro` and `ere_datetimes_impl` is unique to this project, *Super Speedy Syslog Searcher*.
It is licensed under the project top-level MIT License.

Each subproject is deployed as a separate crate:

1. proc-macros must be in their own crate; `ere-macros` and `ere_automator_procmacro` are `proc-macro` crates.
2. it takes a long time to compile `ere_datetimes_impl` (tens of minutes). So `ere_datetimes_impl` is put in its own crate to avoid recompiling when unrelated code is changed.
