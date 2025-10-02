---
applyTo: '**'
---

# GitHub Copilot Instructions for Marty Workspace

This workspace is a Cargo workspace for a language-agnostic monorepo management tool. Follow these guidelines to ensure best practices for Cargo workspaces, maintainability, and code quality.

## Workspace Structure and Best Practices
- Organize crates logically within the workspace, using `members` in `Cargo.toml` to include relevant paths.
- Use workspace-level dependencies in `[workspace.dependencies]` to share common crates and versions across members.
- Ensure each crate has a clear, focused purpose; avoid monolithic crates.
- Follow Cargo workspace conventions: use `workspace = true` for shared metadata like authors, edition, and license.
- Maintain a flat or shallow directory structure for crates to simplify navigation and dependencies.

## Creating New Crates
- When logic is unrelated to an existing crate, create a new crate in the workspace.
- Name new crates descriptively, using kebab-case (e.g., `marty-core`, `marty-cli`).
- Add the new crate to the workspace's `members` array in the root `Cargo.toml`.
- Include necessary dependencies and features, preferring workspace dependencies where possible.

## Linting and Code Quality
- Always adhere to all lints, including warnings. Run `cargo clippy` regularly and fix all issues.
- Enforce strict linting in `Cargo.toml` with `[lints]` sections, enabling pedantic and nursery lints where appropriate.
- Use tools like `rustfmt` for consistent formatting.

## Type Safety and Performance
- Prioritize type safety: use strong typing, avoid `unsafe` code unless absolutely necessary, and leverage Rust's ownership system.
- Prefer enums over strings for representing fixed sets of values (e.g., hook types, languages, command types) to improve type safety, enable exhaustive matching, and catch errors at compile time rather than runtime.
- For FFI boundaries (especially WASM), ensure types are FFI-safe by using appropriate `#[repr(...)]` attributes on enums and structs.
- When conflicts arise (e.g., between safety and performance), prefer the more type-safe option. If performance is critical, document trade-offs and consider alternatives like `unsafe` only after profiling.
- Optimize for performance by choosing efficient data structures, minimizing allocations, and using async where beneficial for I/O-bound tasks.
- Profile code with tools like `cargo flamegraph` or `criterion` to guide optimizations.
- Do not use unsafe blocks

## General Guidelines
- Write idiomatic Rust code, following the Rust API guidelines.
- Document public APIs with `rustdoc` comments.
- Ensure cross-platform compatibility, as this is a language-agnostic tool.
- Test thoroughly: include unit tests, integration tests, and fuzzing where applicable.
- Keep commits atomic and well-described.

These instructions ensure the workspace remains scalable, maintainable, and aligned with Rust best practices.