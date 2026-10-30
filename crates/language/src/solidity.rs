#![cfg(test)]
use super::*;
use ast_grep_core::{AstGrep, Pattern};

fn test_match(query: &str, source: &str) {
  use crate::test::test_match_lang;
  test_match_lang(query, source, Solidity);
}

fn test_non_match(query: &str, source: &str) {
  use crate::test::test_non_match_lang;
  test_non_match_lang(query, source, Solidity);
}

#[test]
fn test_solidity_str() {
  test_match("pragma solidity 0.8.28;", "pragma solidity 0.8.28;");
  test_match(
    r#"import { Test } from "forge-std/Test.sol";"#,
    r#"import { Test } from "forge-std/Test.sol";"#,
  );
  test_non_match("pragma solidity 0.8.28;", "pragma solidity 0.8.26;");
  test_non_match(
    r#"import { Test } from "forge-std/Test.sol";"#,
    r#"import { console } from "forge-std/Test.sol";"#,
  );
}

#[test]
fn test_solidity_pattern() {
  test_match(
    r#"import { $A } from "forge-std/Test.sol";"#,
    r#"import { Test } from "forge-std/Test.sol";"#,
  );
  test_match(
    r#"import { $$$ } from "forge-std/Test.sol";"#,
    r#"import { Test, console } from "forge-std/Test.sol";"#,
  );
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
  use crate::test::test_replace_lang;
  test_replace_lang(src, pattern, replacer, Solidity)
}

#[test]
fn test_solidity_replace() {
  let ret = test_replace(
    r#"import { Test } from "forge-std/Test.sol";"#,
    "Test",
    "console",
  );
  assert_eq!(ret, r#"import { console } from "forge-std/Test.sol";"#);
}

// tree-sitter-solidity 1.2.13 gives `array_access` no precedence. An index
// after a binary, unary, or ternary operator applies to the whole operator
// expression, so `a + b[i]` parses as `(a + b)[i]`.

fn parse_expression(expression: &str) -> AstGrep<StrDoc<Solidity>> {
  Solidity.ast_grep(format!(
    "contract C {{ function f() public {{ y = {expression}; }} }}"
  ))
}

fn field_text(expression: &str, kind: &str, field: &str) -> String {
  let root = parse_expression(expression);
  let node = root.root().dfs().find(|n| n.kind() == kind);
  let node = node.unwrap_or_else(|| panic!("no {kind} in `{expression}`"));
  let child = node.field(field);
  let child = child.unwrap_or_else(|| panic!("no {kind}.{field} in `{expression}`"));
  child.text().to_string()
}

fn captured(expression: &str, pattern: &str, var: &str) -> String {
  let root = parse_expression(expression);
  let found = root.root().find(Pattern::new(pattern, Solidity));
  let found = found.unwrap_or_else(|| panic!("`{pattern}` does not match `{expression}`"));
  let node = found.get_env().get_match(var).expect("should capture");
  node.text().to_string()
}

#[test]
fn test_solidity_index_after_binary_operator() {
  assert_eq!(field_text("a + b[i]", "array_access", "base"), "b");
  assert_eq!(field_text("a == b[i]", "array_access", "base"), "b");
  assert_eq!(captured("a + b[i]", "$A + $B", "B"), "b[i]");
  assert_eq!(captured("total + balances[to]", "$M[$K]", "M"), "balances");
}

#[test]
fn test_solidity_index_after_unary_operator() {
  assert_eq!(field_text("-a[i]", "array_access", "base"), "a");
  assert_eq!(
    field_text("delete a[i]", "unary_expression", "argument"),
    "a[i]"
  );
}

#[test]
fn test_solidity_index_after_ternary_operator() {
  assert_eq!(field_text("c ? a : b[i]", "array_access", "base"), "b");
  assert_eq!(captured("c ? a : b[i]", "$C ? $A : $B", "B"), "b[i]");
}

#[test]
fn test_solidity_index_without_operator() {
  assert_eq!(field_text("a[i] + b", "binary_expression", "left"), "a[i]");
  assert_eq!(field_text("(a + b)[i]", "array_access", "base"), "(a + b)");
}
