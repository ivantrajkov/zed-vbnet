# zed-vbnet

Zed .NET Visual Basic support: VB.NET for the [Zed](https://zed.dev) editor.

- Syntax highlighting, outline, bracket matching, indentation and text objects,
  using [tree-sitter-vb-dotnet](https://github.com/CodeAnt-AI/tree-sitter-vb-dotnet).
- Language server: [vb-ls](https://www.nuget.org/packages/vb-ls/), a Roslyn
  language server fork with Visual Basic enabled. It provides go to definition
  across files and projects (including decompiled framework sources), find
  references, rename, hover, completion, diagnostics, workspace symbols and
  semantic highlighting.

## Requirements

- .NET 10 SDK (`dotnet` on `PATH`).
- For installing as a dev extension: Rust via `rustup` and a C toolchain
  (`sudo apt install build-essential` on Ubuntu/Debian).

## Language server lookup

The extension starts `vb-ls` from the first place it finds:

1. `lsp."vb-ls".binary.path` in your Zed settings.
2. `vb-ls` on `PATH` (e.g. `dotnet tool install --global vb-ls`).
3. Otherwise it downloads the latest `vb-ls` package from NuGet and runs it with
   `dotnet`.

The default arguments are `--stdio --autoLoadProjects`; the server finds the
`.sln`/`.slnx`/`.vbproj` files in your workspace.

## Recommended settings

The grammar does not expose VB keywords as syntax nodes, so keywords are coloured
by the language server's semantic tokens. Turn them on for VB.NET:

```json
{
  "languages": {
    "VB.NET": {
      "semantic_tokens": "combined"
    }
  }
}
```

Custom server binary or arguments:

```json
{
  "lsp": {
    "vb-ls": {
      "binary": {
        "path": "/path/to/vb-ls",
        "arguments": ["--stdio", "--autoLoadProjects", "--logLevel", "Information"]
      }
    }
  }
}
```

## Development

```sh
cargo build --release --target wasm32-wasip2   # check the extension compiles
```

In Zed: command palette → `zed: install dev extension` → select this folder.
Logs: `zed: open log`.

## Known limitations

- Until `vb-ls` finishes loading the project, keywords (`Sub`, `If`, `End` …)
  are not coloured; types, names, strings, numbers and comments are.
- Block indentation is pattern-based (`config.toml`); auto-properties and
  interface members without a body indent the next line.
- The grammar does not yet parse every VB.NET construct (for example `Handles`,
  `Operator`, XML literals). Unparsed regions fall back to semantic highlighting.
