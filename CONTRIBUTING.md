# Contributing

This document outlines some helpful guidelines if you wish to contribute to the code.

## CI Requirements

- We enforce rustfmt in CI. Try setting it up to format on save in your IDE (links for
  [vscode][vscode fmt], [rustrover][rustrover fmt], [neovim][neovim fmt]).
- We enforce that `cargo clippy` passes in CI. Be sure to fix any warnings.
- We also enforce that tests pass.

## Project Structure

Code broadly useful to the ecosystem should go under the `crates/` directory.
Application-specific code should go under the `apps/` directory. It is advisable to
treat your code as application specific initially, and later migrate it to `crates/`
once you are more confident about how to use it outside your application specific
context.

## Style Guide

General guidance:

- Key types, and any pub re-exports should always be at the top of the module.
  Readers should be able to get the big picture of what is happening from the top
  of the file.
- Avoid over-commenting code. Only comment when you wouldn't otherwise know what a type
  or function is doing based on its name and arguments alone.
- DO comment high level details that help a newcomer understand the organization of the
  codebase. Typically these are module-level doc comments that explain the scope and/or
  responsiblity of the module.
- Limit your use of third party dependencies. Only bring it in if it greatly reduces the
  amound of code. In particular, avoid esoteric macro crates unless they are widely
  used. Generally you can do the same thing easily with a quick `macro_rules`.
- Leverage cargo's [workspace inheritance].
- Avoid IO when possible. For example, instead of having a function called
  `spawn_from_file(path: Path)`, consider a `spawn_from_data(data: &[u8])`. This is
  broadly useful for running the code in more contexts (including sandboxed ones), as
  well as atypical environments (such as on a server, or in tests). In other words,
  write code [sans-io](sansio).
- If a function might be called a lot, avoid allocating. For example instead of
  `create_things() -> Vec<Thing>`, consider if `create_things(&mut Vec<Thing>)` would
  be better - it allows the caller to reuse the vector. Don't take this rule too far
  though, if it looks like it is going to overcomplicate the API.
- If a function is *almost* pure, try to make it pure ([Carmack agrees][carmack style]).
- If a function is only used in one place and is less than 25 lines of code, it should
  be inlined ([Carmack agrees][carmack style]).
- Don't associate functions to structs *just because*. Module-level functions are
  perfectly fine and are often preferred because they have less indentation.
- Don't go crazy introducing generics and traits everywhere. `Box<dyn T>` or an enum is
  often fine, and usually a trait is overkill to begin with.
- Unless there is some specific invariant you need to protect, prefer directly exposing
  fields of structs via `pub` over getters and setters.
- Prefer [newtypes][newtype] for stronger type safety whenever possible. Types document
  code better than comments.

[workspace inheritance]: https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table
[vscode fmt]: https://stackoverflow.com/a/54665086
[rustrover fmt]: https://www.jetbrains.com/help/rust/rustfmt.html#rustfmt-on-save
[neovim fmt]: https://www.jvt.me/posts/2022/03/01/neovim-format-on-save/
[sansio]: https://sans-io.readthedocs.io/
[carmack style]: https://cbarrete.com/carmack.html
[newtype]: https://rust-lang.github.io/api-guidelines/type-safety.html#c-newtype
