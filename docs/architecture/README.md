# Architecture documentation

This section explains how the current repository is wired and where key responsibilities live.

## Start here

- [design.md](design.md): runtime behavior and ownership of the main workflow.
- [repository-map.md](repository-map.md): crate-by-crate repository map and code entry points.

## Use this section when

- you need to identify the owning crate for a change
- you want to understand the current request path from server to domain to persistence
- you need the repository-level reference semantics before changing deduplication or persistence
