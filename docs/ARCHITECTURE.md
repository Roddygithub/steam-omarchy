# Architecture

Vite builds a React/TypeScript UI embedded by Tauri 2. Rust IPC commands bridge
catalog discovery, validated Steam actions, preferences and theme reading.
`catalog.rs` handles local metadata and bounded artwork reads; `theme.rs`
validates theme roles; `main.rs` registers IPC and owns preference persistence.
React owns navigation and view state; CSS consumes semantic color tokens.

Linux uses GTK3 and WebKitGTK, not Qt/QML. The CSP restricts external content.
The backend is a trust boundary: validate all frontend arguments and filesystem
input. System libraries are dynamically linked and retain their own licenses.
No private environment inventory or design captures are necessary to build.
See BUILDING.md for checks and THIRD-PARTY-NOTICES.md for compliance limits.
