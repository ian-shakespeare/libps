# Contributing to libPS

We'd love to have you contribute to libPS!

This document is a quick helper to get you going.

## Getting Started

libPS is a spec-compliant PostScript interpreter built with Rust. If you are unfamiliar PostScript, the language reference is a good starting point:

* [PostScript Language Reference, 3rd Edition](https://www.adobe.com/jp/print/postscript/pdfs/PLRM.pdf)

If you are new to Rust, the following books are recommended reading:

* Jim Blandy et al. [Programming Rust, 2nd Edition](https://www.oreilly.com/library/view/programming-rust-2nd/9781492052586/). 2021
* Steve Klabnik and Carol Nichols. [The Rust Programming Language](https://doc.rust-lang.org/book/#the-rust-programming-language). 2022

To build and run `libPS` cli: 

```shell 
cargo run --package libps --bin libps
```

Run tests:

```console
cargo test
```

Test coverage report:

```
cargo llvm-cov --html
```

> [!NOTE]
> Generation of coverage report requires [llvm-cov](https://lib.rs/crates/cargo-llvm-cov) binary to be installed.
> You can install it with `cargo install cargo-llvm-cov`

## Finding things to work on

The issue tracker has issues tagged with [good first issue](https://github.com/ian-shakespeare/libps/issues?q=is%3Aissue%20state%3Aopen%20label%3A%22good%20first%20issue%22),
which are considered to be things to work on to get going. If you're interested in working on one of them, comment on the issue tracker, and we're happy to help you get going.

## Submitting your work

Fork the repository and open a pull request to submit your work.

The CI checks for formatting, Clippy warnings, and test failures so remember to run the following before submitting your pull request:

* `cargo fmt` and `cargo clippy` to keep the code formatting in check.
* `make` to run the test suite.

**Keep your pull requests focused and as small as possible, but not smaller.** IOW, when preparing a pull request, ensure it focuses on a single thing and that your commits align with that. For example, a good pull request might fix a specific bug or a group of related bugs. Or a good pull request might add a new feature and test for it. Conversely, a bad pull request might fix a bug, add a new feature, and refactor some code.

**The commits in your pull request tell the story of your change.** Break your pull request into multiple commits when needed to make it easier to review and ensure that future developers can also understand the change as they are in the middle of a `git bisect` run to debug a nasty bug. A developer should be able to reconstruct the intent of your change and how you got to the end-result by reading the commits. To keep a clean commit history, make sure the commits are _atomic_:

* **Keep commits as small as possible**. The smaller the commit, the easier it is to review, but also easier `git revert` when things go bad.
* **Don't mix logic and cleanups in same commit**. If you need to refactor the code, do it in a commit of its own. Mixing refactoring with logic changes makes it very hard to review a commit.
* **Don't mix logic and formatting changes in same commit**. Resist the urge to fix random formatting issues in the same commit as your logic changes, because it only makes it harder to review the commit.
* **Write a good commit message**. You know your commit is atomic when it's easy to write a short commit message that describes the intent of the change.

To produce pull requests like this, you should learn how to use Git's interactive rebase (`git rebase -i`).

For a longer discussion on good commits, see Al Tenhundfeld's [What makes a good git commit](https://www.simplethread.com/what-makes-a-good-git-commit/), for example.

## Adding Third Party Dependencies

libPS is a zero dependency library. Third party dependencies are strictly forbidden. Should a library prove necessary, create an issue to discuss vendoring in the source code.
