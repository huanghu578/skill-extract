# skill-extract

A configuration-driven framework for extracting structured key-value pairs from
natural language instructions.

Skills are defined in TOML. Extraction rules are regular expressions with a
fixed `(?P<value>...)` named capture group. Keys can be any Unicode string —
Chinese, Japanese, emoji, anything.

Designed as a foundation layer for software that needs to turn free-form
instructions into structured parameters: agent tool dispatch, task routers,
DSL front-ends, and similar systems.

---

## Table of contents

- [Why](#why)
- [Core concepts](#core-concepts)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Skill file format](#skill-file-format)
- [Output format](#output-format)
- [Built-in post-processors](#built-in-post-processors)
- [Output schema](#output-schema)
- [Library API](#library-api)
- [CLI](#cli)
- [Extending the framework](#extending-the-framework)
- [Examples](#examples)
- [Design notes](#design-notes)
- [Project layout](#project-layout)
- [Building and testing](#building-and-testing)
- [Publishing](#publishing)
- [License](#license)

---

## Why

Most "parse this instruction" tasks are ad-hoc: a regex here, a `split()` there,
a hand-rolled state machine when things get complicated. Every project ends up
reinventing the same wheel, and every domain expert who knows *what* should be
extracted has to ask a programmer to change code.

`skill-extract` separates the two concerns:

- **The rules** live in TOML files. Anyone can write or edit them.
- **The engine** compiles those rules, runs them against text, and returns
  structured JSON.

You get:

- **Config-driven extraction** — define what to extract without touching Rust.
- **Unicode-native keys** — `工作文件夹`, `出力ファイル`, `répertoire` all work.
- **Structured output** — JSON with `values`, `missing`, and `errors`.
- **Trigger-based routing** — one instruction can select the right skill.
- **Schema validation** — declare expected types, get errors when they don't match.
- **Stable ASCII aliases** — every key can have an optional `key_en` alias.

---

## Core concepts

| Concept | What it is |
|---------|-----------|
| **Skill** | A compiled TOML configuration. It knows a set of extractors. |
| **Extractor** | One rule: a key, a regex, and optional flags/post-processing. |
| **Trigger** | A substring used to route an instruction to a skill. |
| **ExtractResult** | The output: `values`, `missing`, `errors`. |
| **Registry** | A collection of skills, loaded from a directory, with routing. |
| **Post-processor** | A transformation applied to a value after regex matching. |
| **Output schema** | Optional type declarations for validation. |

---

## Installation

### As a library

```toml
[dependencies]
skill-extract = "0.1.0"
```

### As a CLI

```bash
cargo install skill-extract
```

---

## Quick start

### 1. Write a skill file

`skills/dropper_task.toml`:

```toml
name = "dropper_task"
description = "Extract suspension_type and I/O files for dropper calculation"
triggers = ["小dropper", "吊弦"]

[[extractors]]
key = "suspension_type"
key_en = "suspension_type"
pattern = 'suspension_type[:：]\s*(?P<value>[A-Za-z0-9+\-_/]+)'
required = true
multi = false
postprocess = ["trim", "strip_quotes"]

[[extractors]]
key = "输入文件"
key_en = "input_file"
pattern = '输入文件[:：]\s*(?P<value>[^\s，,。；;、]+\.xlsx)'
required = true
multi = false
postprocess = ["trim", "strip_quotes", "normalize_path"]

[[extractors]]
key = "输出文件"
key_en = "output_file"
pattern = '输出文件[:：]\s*(?P<value>[^\s，,。；;、]+\.xlsx)'
required = true
multi = false
postprocess = ["trim", "strip_quotes", "normalize_path"]

[output_schema]
suspension_type = { type = "string", required = true }
input_file = { type = "path", required = true }
output_file = { type = "path", required = true }
```

### 2. Use it

**Library:**

```rust
use skill_extract::Skill;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let skill = Skill::from_file("skills/dropper_task.toml")?;
    let text = "小dropper，请计算 suspension_type: THJ70+CTA85 \
                输入文件: a.xlsx 输出文件: b.xlsx";
    let result = skill.extract(text);
    println!("{}", result.to_json());
    Ok(())
}
```

**Registry:**

```rust
use skill_extract::SkillRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = SkillRegistry::load_dir("skills")?;
    if let Some(result) = registry.extract_routed("小dropper，请计算...") {
        println!("{}", result.to_json());
    }
    Ok(())
}
```

**CLI:**

```bash
skill-extract skills/dropper_task.toml "suspension_type: THJ70+CTA85 输入文件: a.xlsx 输出文件: b.xlsx"
```

---

## Skill file format

### Top-level fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Unique skill identifier. |
| `description` | string | no | Human-readable description. |
| `triggers` | string[] | no | Substrings used for routing. |
| `extractors` | table[] | yes | At least one extraction rule. |
| `output_schema` | table | no | Optional type validation. |

### Extractor fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `key` | string | required | Output field name. Any Unicode. |
| `key_en` | string | — | Optional ASCII alias. |
| `pattern` | string | required | Regex; **must** contain `(?P<value>...)`. |
| `required` | bool | `false` | If true and not found, goes into `missing`. |
| `multi` | bool | `true` | If true, collect all matches; else first only. |
| `postprocess` | string[] | `[]` | Ordered pipeline of transformations. |

### Why `(?P<value>...)`?

If the key name itself were used as the capture group name, Unicode keys
(Chinese, Japanese) would depend on regex engine support. A fixed `value` group
decouples the regex from the output key and keeps the pattern portable.

---

## Output format

`ExtractResult` serializes to JSON:

```json
{
  "values": {
    "suspension_type": ["THJ70+CTA85"],
    "输入文件": ["整体吊弦原始数据输入模板.xlsx"],
    "input_file": ["整体吊弦原始数据输入模板.xlsx"],
    "输出文件": ["计算结果.xlsx"],
    "output_file": ["计算结果.xlsx"]
  },
  "missing": [],
  "errors": []
}
```

| Field | Meaning |
|-------|---------|
| `values` | Successfully extracted keys. Both `key` and `key_en` appear. |
| `missing` | Keys with `required = true` that were not found. |
| `errors` | Schema validation errors (e.g. type mismatch). |

`is_complete()` returns `true` when both `missing` and `errors` are empty.

---

## Built-in post-processors

| Name | Effect |
|------|--------|
| `trim` | Strip leading/trailing whitespace. |
| `lowercase` | Convert to lowercase. |
| `uppercase` | Convert to uppercase. |
| `strip_quotes` | Remove paired `"..."` or `'...'`. |
| `normalize_path` | Replace `\` with `/`. |
| `replace` | Regex replace: `{ pattern = "...", replacement = "..." }` |
| `filter_out` | Drop values matching a regex: `{ pattern = "..." }` |

Post-processors run in the order they appear in `postprocess`.

**Example:**

```toml
postprocess = [
  "trim",
  "strip_quotes",
  "normalize_path",
  { replace = { pattern = "^file://", replacement = "" } },
]
```

---

## Output schema

The optional `[output_schema]` section declares expected types. After
extraction, the framework validates the result and reports mismatches in
`errors`.

```toml
[output_schema]
suspension_type = { type = "string", required = true }
input_file = { type = "path", required = true }
output_file = { type = "path", required = true }
count = { type = "int", required = false }
ratio = { type = "float", required = false }
enabled = { type = "bool", required = false }
```

| Type | Validation |
|------|------------|
| `string` | Always passes. |
| `path` | Non-empty. |
| `int` | Parses as `i64`. |
| `float` | Parses as `f64`. |
| `bool` | One of `true`, `false`, `0`, `1`. |

---

## Library API

### `Skill`

```rust
impl Skill {
    /// Load from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self>;

    /// Load from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self>;

    /// Build from a parsed config.
    pub fn from_config(config: SkillConfig) -> Result<Self>;

    /// Check whether any trigger matches `text`.
    pub fn matches(&self, text: &str) -> bool;

    /// Extract all configured keys from `text`.
    pub fn extract(&self, text: &str) -> ExtractResult;

    /// Extract and return JSON directly.
    pub fn extract_to_json(&self, text: &str) -> String;
}
```

### `ExtractResult`

```rust
impl ExtractResult {
    pub fn is_complete(&self) -> bool;
    pub fn to_json(&self) -> String;
    pub fn to_json_compact(&self) -> String;
    pub fn get_first(&self, key: &str) -> Option<&str>;
}
```

### `SkillRegistry`

```rust
impl SkillRegistry {
    pub fn new() -> Self;
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self>;
    pub fn register(&mut self, skill: Skill);
    pub fn get(&self, name: &str) -> Option<&Skill>;
    pub fn list(&self) -> Vec<&str>;
    pub fn route(&self, text: &str) -> Option<&Skill>;
    pub fn extract_routed(&self, text: &str) -> Option<ExtractResult>;
    pub fn extract_all_matching(&self, text: &str)
        -> HashMap<String, ExtractResult>;
    pub postprocessors: PostProcessorRegistry;
}
```

### Errors

`SkillError` is a structured enum; every variant carries context:

| Variant | When |
|---------|------|
| `ConfigNotFound { path, cwd }` | The TOML file does not exist. |
| `Io { path, source }` | Read error. |
| `Toml { path, source }` | TOML parse error. |
| `InvalidPattern { key, reason }` | Regex failed to compile. |
| `MissingValueGroup { key }` | Pattern lacks `(?P<value>...)`. |
| `MissingRequired { keys }` | Schema validation: required key missing. |
| `TypeMismatch { key, expected, got }` | Schema validation: type mismatch. |
| `InvalidConfig(String)` | Structural config error (e.g. empty extractors). |

---

## CLI

```
skill-extract <skill.toml> <text>
skill-extract --dir <skills_dir> --route <text>
skill-extract --dir <skills_dir> --list
skill-extract <skill.toml> --stdin
```

### Examples

```bash
# Single skill
skill-extract skills/dropper_task.toml \
  "suspension_type: THJ70+CTA85 输入文件: a.xlsx 输出文件: b.xlsx"

# Directory + routing
skill-extract --dir skills --route "小dropper，请计算..."

# List all skills in a directory
skill-extract --dir skills --list

# Read text from stdin
echo "suspension_type: THJ70+CTA85" | skill-extract skills/dropper_task.toml --stdin
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success; all required keys present. |
| 1 | Configuration or I/O error. |
| 2 | Extraction incomplete (missing required keys or schema error). |
| 3 | No skill matched the input (route mode only). |

Exit codes make the CLI easy to embed in shell scripts and agent frameworks.

---

## Extending the framework

### Custom post-processors

Implement the `PostProcessor` trait and register it:

```rust
use skill_extract::{PostProcessor, SkillRegistry};

struct Basename;

impl PostProcessor for Basename {
    fn name(&self) -> &str { "basename" }

    fn process(&self, value: &str) -> Option<String> {
        std::path::Path::new(value)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
    }
}

let mut registry = SkillRegistry::new();
registry.postprocessors.register(Basename);
```

The registry stores processors by name. To invoke them programmatically:

```rust
if let Some(p) = registry.postprocessors.get("basename") {
    let out = p.process("/tmp/result.csv");
    assert_eq!(out.as_deref(), Some("result.csv"));
}
```

> **Note:** The current `Skill::extract` pipeline only executes built-in
> post-processors. Custom processors are registered and callable, but not yet
> wired into extraction. Wiring them in requires passing the registry (or a
> callback) into `extract`.

### Alternative output formats

`ExtractResult` implements `Serialize`, so any serde-compatible format works:

```rust
let yaml = serde_yaml::to_string(&result)?;
let toml = toml::to_string(&result)?;
```

### Custom entry points

`Skill` and `SkillRegistry` are plain library types. You can wrap them in:

- An MCP server exposing each skill as a tool.
- An HTTP handler: `POST /extract/{skill_name}`.
- A gRPC service for multi-language clients.
- A WASM module for browser-side extraction.

None of these require changes to the core.

---

## Examples

Three runnable examples live in `examples/`:

| Example | What it shows |
|---------|---------------|
| `01_library_usage` | Minimal usage: load one skill, extract, print JSON. |
| `02_registry_routing` | Multiple skills, trigger routing, all-matching mode. |
| `03_custom_postprocessor` | Implementing and registering a `PostProcessor`. |

Run them:

```bash
cargo run --example 01_library_usage
cargo run --example 02_registry_routing
cargo run --example 03_custom_postprocessor
```

---

## Design notes

**Why `(?P<value>...)`?**
A fixed capture group name decouples the regex from the output key. Unicode key
names stay portable across regex engines.

**Why JSON output?**
JSON is the de facto interchange format for LLM tool calls, agent frameworks,
and cross-language pipelines. TOML is better for *human-authored input* — which
is why skills are TOML but results are JSON.

**Why `key_en`?**
Downstream code often prefers a stable ASCII identifier. `key` is for humans,
`key_en` is for code. Both point to the same values.

**Why is the engine compile-time strict?**
All regex compilation and config parsing happens in `Skill::from_config`. Once
a `Skill` exists, `extract()` cannot fail for configuration reasons. All
"expected failures" (missing keys, type mismatches) land in the `missing` /
`errors` fields of `ExtractResult`.

**Why are `missing` and `errors` separate?**
`missing` means "required but not found". `errors` means "found but does not
match the schema". They require different handling by the caller.

---

## Project layout

```
skill-extract/
├── Cargo.toml
├── README.md
├── .gitignore
├── skills/
│   ├── dropper_task.toml
│   ├── beam_task.toml
│   └── extract_paths.toml
├── examples/
│   ├── 01_library_usage.rs
│   ├── 02_registry_routing.rs
│   └── 03_custom_postprocessor.rs
├── src/
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── schema.rs
│   ├── skill.rs
│   ├── registry.rs
│   ├── postprocess.rs
│   └── bin/
│       └── skill-extract.rs
└── tests/
    ├── extraction.rs
    ├── registry.rs
    └── cli.rs
```

---

## Building and testing

```bash
cargo build
cargo build --examples
cargo test
```

22 tests across three suites:

- `tests/extraction.rs` — 11 tests for extraction, schema, Unicode keys.
- `tests/registry.rs` — 7 tests for registry, routing, custom processors.
- `tests/cli.rs` — 4 tests for CLI modes and exit codes.

---

## Publishing

```bash
cargo publish
```

The crate ships as both a library and a binary. Library users get the full
API; CLI users get the `skill-extract` executable via `cargo install`.

---

## License

MIT or Apache-2.0