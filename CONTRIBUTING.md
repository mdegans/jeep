# Contributing (Philosophy):

* **Everything outside this crate is broken**, including potentially the kernel,
so input is *never* trusted, everything is fallible, and nothing should *ever*
cause a [`panic`]. Ideally that possibility is ruled out at compile time.
* **Everything should be optional**, so [`serde`] support, for example, is
off by default. In the future, this crate might work in `no_std` environments.
If you add a crate in a PR, please `[cfg(feature = "foo")]` all it's use.
* **[`unsafe`](https://doc.rust-lang.org/nomicon/) is avoided** when there's a
good alternative, but there are a few uses in the [`frame`](src/frame.rs) module. Third
party audits are welcome.
* **Don't Repeat Yourself**, but at the same time, the various [`events`](src/events) are so
different that there is no perfect abstraction for all events. A pretty API
for DateTime and Doors is better than trying to make one trait or struct fit
them all. Using re-exported external types is fine where appropriate.
* **100% test coverage isn't unrealistic**, so let's make it happen.
If you find a bug, write a test, fix the code, and submit a PR.
* **Everybody is learning** all the time. Don't talk down to others. Be humble.
There is always somebody who knows more you.