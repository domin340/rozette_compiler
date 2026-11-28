# Rozette AST Roadmap

This roadmap follows the syntax examples in `examples/`, ordered from the MVP foundations to the full language surface.
It is intentionally AST/parser focused: each phase must add stable AST nodes, lexer support, parsing behavior, spans, and focused tests.

Status legend:

- [x] implemented
- [~] partially implemented
- [ ] not implemented

## Foundation and repository conventions

- [x] Use `Span` (`u32`, `u32`) for source ranges.
- [x] Intern identifiers and literal text with `Symbol`.
- [ ] Add parser diagnostics instead of `panic!` for invalid syntax.

## Phase 0 - Lexical foundation

Source: `examples/01_basics.roze`, `examples/syntax/tokens.txt`

- [~] Comments
  - [x] line comments
  - [x] doc comments
  - [x] file doc comments
  - [x] inline/block comments
  - [ ] attach comments to AST items or retain them in a side table
- [x] newlines
- [x] semicolons
- [ ] decide and document comment ownership and preserving source trivia

Exit criteria:

- Comments are tokenized with accurate spans and comment kind.
- AST consumers can opt into comments or ignore trivia consistently.

## Phase 1 - Variables, literals, operators

Source: `examples/01_basics.roze`

- [x] Keywords: `let`, `mut`, `as`
- [x] Literals
  - [x] numbers
  - [x] double-quoted strings
  - [x] `true` and `false`
- [x] identifiers and underscore patterns
- [x] arithmetic, comparison, logical, and assignment operators
- [x] binary and unary expressions
- [x] assignment and compound assignment expressions
- [x] casts
- [x] immutable and mutable `let` bindings
- [x] semicolon and semicolon-less statements
- [x] equal equal and not equal operator lexed and represented in ast
- [x] lex lifetimes

Exit criteria:

- add parser tests for precedence, casts, and invalid assignments
- `examples/01_basics.roze` parses without parser `todo!` or unsupported-token failures.

## Phase 2 - Functions and calls

Source: `examples/02_functions.roze`

- [x] `fn` items
- [x] function identifiers
- [x] basic function parameters
  - [x] identifiers
  - [x] mutability
  - [x] optional parameter types
- [~] function return types
  - [x] never and unit types
  - [x] path types
  - [ ] tuple types
- [x] block bodies
- [x] implicit and explicit return expressions
- [x] function calls and argument lists
- [ ] `StmtKind::Expr`/return value rules for block results

Required AST additions/completions:

- [ ] `TyPatKind::Tuple`
- [ ] `TyPatKind::Any`
- [ ] Complete tuple function return types such as `(number, string)`
- [ ] Replace tuple type `todo!` with a tested parser
- [ ] Represent implicit block return unambiguously

Exit criteria:

- `examples/02_functions.roze` parses, including tuple types and `!`.

## Phase 3 - Modules and declarations

Source: `examples/03_modules.roze`

- [~] visibility
  - [x] private visibility
  - [x] `pub`
  - [x] restricted visibility shape
- [~] declarations
  - [x] use trees
  - [x] constants
  - [x] static variables
  - [x] modules
  - [ ] `ItemKind::Enum`, `ItemKind::Type`, `ItemKind::Impl`
- [x] path expressions and path types
- [ ] use-tree nested imports and globs must be covered by parser tests

Required AST additions/completions:

- [ ] Public exports for `Item`, `TyPat`, `Block`, `Local`, `Fn`, `Mod`, and related nodes
- [ ] Validation for invalid visibility combinations
- [ ] Module-level item parser for every item kind

Exit criteria:

- `examples/03_modules.roze` parses, including nested modules and aliased/glob use syntax.

## Phase 4 - Control flow and ranges

Source: `examples/04_control_flow.roze`

- [~] `if` expressions
  - [x] one-branch `if`
  - [x] `else` block parsing
- [~] loops
  - [ ] `loop`
  - [ ] `while`
  - [x] `for`
- [~] loop control
  - [x] `break` expression AST shape
  - [x] `continue` expression AST shape
  - [ ] `loop` expressions
- [x] inclusive/exclusive range AST

Required AST additions/completions:

- [ ] `ExprKind::Loop`
- [ ] `ExprKind::While`
- [ ] `ExprKind::Let` for `if let` and `while let`
- [ ] Add precedence and block-result tests for control expressions

Exit criteria:

- `examples/04_control_flow.roze` and `examples/07_loops.roze` parse.

## Phase 5 - Blocks, tuples, arrays, tables, and indexing

Source: `examples/05_collections.roze`

- [ ] `ExprKind::Tuple`
- [ ] `ExprKind::Paren`
- [ ] `ExprKind::Unit`
- [ ] `ExprKind::Block`
- [ ] `ExprKind::Array`
- [ ] `ExprKind::Table`
- [ ] `ExprKind::Index`
- [ ] Disambiguate `{ let ... }` blocks from `{ x: 1 }` tables
- [ ] Allow trailing commas in arrays and tables
- [ ] Parse nested arrays, nested tuples, and chained indexes
- [ ] Parse table field access after the field-expression phase

Exit criteria:

- `examples/05_collections.roze` parses completely.
- One-element tuple ambiguity is resolved correctly.

## Phase 6 - Patterns and let-else

Source: `examples/06_patterns.roze`

- [~] identifier and discard patterns
- [ ] path patterns
- [ ] range patterns
- [ ] parenthesized patterns
- [ ] tuple patterns
- [ ] call patterns such as `Some(x)`
- [ ] or-patterns such as `A | B | C`
- [ ] literal patterns
- [ ] table/struct patterns
- [ ] array patterns
- [ ] rest patterns
- [ ] `@` binding patterns
- [ ] `let else` statements
- [ ] guarded match arms

Required AST additions/completions:

- [ ] `PatKind::Path`
- [ ] `PatKind::Range`
- [ ] `PatKind::Paren`
- [ ] `PatKind::Tuple`
- [ ] `PatKind::Call`
- [ ] `PatKind::Or`
- [ ] `PatKind::Lit`
- [ ] `PatKind::Struct`
- [ ] `PatKind::Rest`
- [ ] `PatKind::Array`
- [ ] `PatKind::Satisfies`
- [ ] `StmtKind::LetElse`

Exit criteria:

- Identifier, tuple, or-pattern, range, enum-call, and let-else patterns in `examples/06_patterns.roze` parse.

## Phase 7 - Remaining iterator expressions

Source: `examples/07_loops.roze`

- [ ] `ExprKind::Let` for `if let` and `while let`
- [ ] `ExprKind::For`
- [ ] `ExprKind::While`
- [ ] `while` keyword and `in` keyword coverage
- [ ] Labeled for loops
- [ ] For-loop breaking without values
- [ ] While-loop breaking without values

Exit criteria:

- `examples/07_loops.roze` parses completely.

## Phase 8 - Lambdas and string interpolation

Source: `examples/08_lambdas.roze`

- [ ] Lambda parameters between `|`
- [ ] Lambda parameter type annotations
- [ ] Optional lambda return type
- [ ] Lambda expression body and block body
- [ ] Lambda/table/block disambiguation
- [ ] Capturing syntax needed for AST semantics
- [ ] `ExprKind::Lambda`
- [ ] `ExprKind::StrInterp`
- [ ] Function type patterns such as `fn(number) -> number`
- [ ] Nested interpolation expressions

Required lexer additions:

- [ ] `lambda` pipe handling is already related to `||`, but distinguish pipe tokens
- [ ] interpolation delimiters inside string tokens

Exit criteria:

- `examples/08_lambdas.roze` parses, including closures and nested interpolated strings.

## Phase 9 - Types, enums, implementations, fields, and methods

Source: `examples/10_type_and_impl.roze`

- [ ] `type` keyword
- [ ] `enum` keyword
- [ ] `Self`/`self` handling
- [ ] `ItemKind::Enum`
- [ ] `ItemKind::Type`
- [ ] `ItemKind::Impl`
- [ ] Type aliases
- [ ] Union types with `|`
- [ ] Anonymous table type patterns
- [ ] Block-based and alias-based type syntax
- [ ] `ExprKind::MethodCall`
- [ ] `ExprKind::Field`
- [ ] Struct initialization
- [ ] Implementation items for functions, constants, and statics

Exit criteria:

- `examples/10_type_and_impl.roze` parses completely.

## Phase 10 - Try expressions and struct updates

Source: `examples/10_type_and_impl.roze`

- [ ] `ExprKind::Try` for postfix `?`
- [ ] `ExprKind::Struct`
- [ ] Struct update syntax (`Point { x: 1, ..p }`)
- [ ] Struct shorthand fields where applicable
- [ ] Table versus struct expression disambiguation
- [ ] Postfix precedence for try, call, index, field, and method access

Exit criteria:

- `to_number(text)?`, `Self { x, y }`, and struct updates parse correctly.

## Phase 11 - Match expressions

Source: `examples/06_patterns.roze`, `examples/11_union.roze`

- [ ] `ExprKind::Match`
- [ ] Match arm AST
- [ ] Match arm patterns
- [ ] Default match arms
- [ ] Guarded match arms
- [ ] Match arm expression/result parsing
- [ ] Or-patterns and enum/value patterns
- [ ] Union narrowing examples

Exit criteria:

- `examples/06_patterns.roze` and `examples/11_union.roze` parse completely.

## Phase 12 - Extended patterns and type patterns

Source: `examples/11_union.roze`

- [ ] `PatKind::Struct`
- [ ] `PatKind::Rest`
- [ ] `PatKind::Array`
- [ ] `PatKind::Satisfies`
- [ ] `TyPatKind::Array`
- [ ] `TyPatKind::Fn`
- [ ] `TyPatKind::Lit`
- [ ] `TyPatKind::Struct`
- [ ] `TyPatKind::Table`
- [ ] Exact array patterns
- [ ] Prefix array patterns with rest patterns
- [ ] Tagged enum type modeling

Exit criteria:

- Array, table, struct, literal, function, and satisfies patterns have parser and AST coverage.

## Phase 13 - Modules across files

Source: `examples/12_file_return.roze`

- [ ] Resolve `mod file;` and file modules
- [ ] Parse the example file syntax embedded in comments
- [ ] Explicit module return values
- [ ] `use file1::return_value as value`
- [ ] Cross-file symbol resolution hooks
- [ ] Define whether module return values belong to AST parsing or later semantic resolution

Exit criteria:

- `examples/12_file_return.roze` is parsed and cross-file module names are represented.

## Follow-up language features

- [ ] Attributes before items and blocks
- [ ] Const blocks
- [ ] Unsafe blocks
- [ ] Luau injection/extern blocks
- [ ] Generic types
- [ ] Traits
- [ ] Tagged enums
- [ ] Error recovery and structured diagnostics
- [ ] Visitor implementation for every AST node
- [ ] Humanizer coverage for every AST node
- [ ] AST lowering/semantics pass
- [ ] Lua/Luau transpiler

## Milestone summary

1. [ ] Parser can process `examples/01_basics.roze` through `examples/05_collections.roze`.
2. [ ] Pattern, loop, lambda, and match phases parse their dedicated examples.
3. [ ] Item/type/implementation phases parse `examples/10_type_and_impl.roze`.
4. [ ] Union and cross-file examples parse and feed later semantic passes.
5. [ ] All example files have automated integration coverage.
