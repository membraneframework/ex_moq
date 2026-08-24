# ExMoQ

[![Hex.pm](https://img.shields.io/hexpm/v/ex_moq.svg)](https://hex.pm/packages/ex_moq)
[![API Docs](https://img.shields.io/badge/api-docs-yellow.svg?style=flat)](https://hexdocs.pm/ex_moq)

Rustler bindings for [moq-dev](https://github.com/moq-dev/moq)'s native MoQ implementation.

## Installation

The package can be installed by adding `ex_moq` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:ex_moq, "~> 0.1.0"}
  ]
end
```

Building requires a Rust toolchain (the NIFs are compiled by [rustler](https://hex.pm/packages/rustler))

## Usage

For a simple example streaming and receiving frames, see [examples/publish_and_subscribe.livemd](examples/publish_and_subscribe.livemd)

## Copyright and License

Copyright 2020, [Software Mansion](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=membrane_gemini_plugin)

[![Software Mansion](https://logo.swmansion.com/logo?color=white&variant=desktop&width=200&tag=membrane-github)](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=membrane_gemini_plugin)

Licensed under the [Apache License, Version 2.0](LICENSE)
