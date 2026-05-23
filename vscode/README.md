# Gill Language Support for VS Code

This extension provides early Gill language integration for Visual Studio Code.

## Features

- Syntax highlighting for `.gill` files
- Language snippets for Gill programs
- Hover descriptions for common Gill constructs
- Extension icon for marketplace listing

## Getting Started

1. Open a `.gill` file in VS Code.
2. Run the command `Gill: Open Welcome` from the command palette.
3. Use `gill-program` snippet to scaffold a new program.

## Language Syntax

Gill supports a hybrid style inspired by C, Rust, and Lisp.

Example:

```gill
NAME="printtoconsole"
DESC="Print program"

func main() {
    print.console("Hello");
}
```
