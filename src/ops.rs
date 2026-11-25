//! Rust `Function` impls for the `combinators` module — the handls
//! (handlers) for the `op`/`handl` declarations in `combinators.hnl`.
//!
//! Callback arguments arrive as `{"__arrow__": …}` wire values under the
//! tree-walk/host eval (and as legacy `FunctionValue` JSON under engines
//! that serialize closures that way). All callback-*consuming* ops go
//! through the `apply_callable*`/`call_with_bindings` helpers in
//! `handl_lib_ops::shared`, which dispatch on the value shape:
//! arrow-values are invoked via `ctx.try_call_value`, `FunctionValue`
//! JSON via `eval_expr`. `atoms.yaml` (P-013/T-151) is the interface
//! spec; these impls provide the logic and are registered in an
//! [`OpRegistry`].
//!
//! The function-*producing* ops (`flow`/`compose`/`fanout_fn`/
//! `constantly`/`flip`/`partial`) were removed: they only accepted and
//! returned legacy `FunctionValue` JSON, which no engine can call. The
//! pure-handl `comb.*` surface in the per-arrow `.hnl` files next to
//! this one covers the same operations with real arrows.
//!
//! The interface spec is `atoms.yaml` (P-013/T-151): the `impl:` keys
//! (`fn_pipe`, …) resolve against the host's `combinators_impls()`
//! table; `combinators_op_registry()` below is the same wiring kept for
//! consumers that still use it directly.

use handl_ops::{Op, OpContext, OpError};

use serde_json::Value as JsonValue;

use handl_lib_ops::shared::{
    apply_callable, apply_callable_args, apply_callable2, arr, b, call_with_bindings,
    is_truthy_callable, key_to_string, map_callable_iter, require_array, require_callable,
};

// ===========================================================================
// Helpers
// ===========================================================================

fn require_two_args(args: &[serde_json::Value], expected: usize) -> Result<(), OpError> {
    if args.len() < expected {
        return Err(OpError::TypeMismatch(format!(
            "expected {} args, got {}",
            expected,
            args.len(),
        )));
    }
    Ok(())
}

// ===========================================================================
// Composition combinators
// ===========================================================================

// ── pipe ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Pipe;
impl Op for Pipe {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 2)?;
        let mut value = args[0].clone();
        for fn_arg in &args[1..] {
            value = apply_callable(ctx, fn_arg, &value)?;
        }
        Ok(value)
    }
}

// ── fanout ─────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Fanout;
impl Op for Fanout {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 2)?;
        let value = args[0].clone();
        let inner = value;
        let mut results = Vec::with_capacity(args.len() - 1);
        for fn_arg in &args[1..] {
            let result = apply_callable(ctx, fn_arg, &inner)?;
            results.push(result);
        }
        Ok(arr(&results))
    }
}

// ── converge ───────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Converge;
impl Op for Converge {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 3)?;
        let value = &args[0];
        // The combiner is only invoked when more than one branch produces
        // a result, but validate it eagerly to preserve the historical
        // argument-checking order.
        require_callable(&args[1])?;
        let branch_count = args.len() - 2;
        let mut results = Vec::with_capacity(branch_count);
        let inner = value.clone();
        for branch_arg in &args[2..] {
            let result = apply_callable(ctx, branch_arg, &inner)?;
            results.push(result);
        }
        let combined = if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            let mut acc = results[0].clone();
            for item in &results[1..] {
                acc = apply_callable2(ctx, &args[1], &acc, item)?;
            }
            acc
        };
        Ok(combined)
    }
}

// ===========================================================================
// Control flow
// ===========================================================================

// ── choice ─────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Choice;
impl Op for Choice {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 4)?;
        let value = &args[0];
        let inner = value.clone();
        let is_true = is_truthy_callable(ctx, &args[1], &inner)?;
        let branch = if is_true { &args[2] } else { &args[3] };
        apply_callable(ctx, branch, &inner)
    }
}

// ── when / unless ──────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct When;
impl Op for When {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 3)?;
        let value = &args[0];
        let inner = value.clone();
        if is_truthy_callable(ctx, &args[1], &inner)? {
            apply_callable(ctx, &args[2], &inner)
        } else {
            Ok(value.clone())
        }
    }
}

#[derive(Clone, Copy)]
pub struct Unless;
impl Op for Unless {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 3)?;
        let value = &args[0];
        let inner = value.clone();
        if is_truthy_callable(ctx, &args[1], &inner)? {
            Ok(value.clone())
        } else {
            apply_callable(ctx, &args[2], &inner)
        }
    }
}

// ===========================================================================
// Pattern matching
// ===========================================================================

fn match_pattern(
    pattern: &JsonValue,
    value: &JsonValue,
    out: &mut Vec<(String, JsonValue)>,
) -> Result<bool, OpError> {
    if let Some(obj) = pattern.as_object() {
        if let Some(type_name) = obj.get("type").and_then(|v| v.as_str()) {
            match type_name {
                "wildcard" => return Ok(true),
                "literal" => {
                    let expected = pattern
                        .get("value")
                        .ok_or_else(|| OpError::Other("literal pattern missing `value`".into()))?;
                    return Ok(expected == value);
                }
                "identifier" => {
                    let name = pattern
                        .get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            OpError::Other("identifier pattern missing `name`".into())
                        })?;
                    out.push((name.to_string(), value.clone()));
                    return Ok(true);
                }
                "array" => {
                    let patterns = pattern
                        .get("patterns")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| OpError::Other("array pattern missing `patterns`".into()))?;
                    let arr = match value.as_array() {
                        Some(a) => a,
                        None => return Ok(false),
                    };
                    for (i, child_pattern) in patterns.iter().enumerate() {
                        let child = arr.get(i).unwrap_or(&JsonValue::Null);
                        if !match_pattern(child_pattern, child, out)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                "object" => {
                    let entries = pattern
                        .get("entries")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| OpError::Other("object pattern missing `entries`".into()))?;
                    let obj = match value.as_object() {
                        Some(o) => o,
                        None => return Ok(false),
                    };
                    for entry in entries {
                        let arr = entry
                            .as_array()
                            .ok_or_else(|| OpError::Other("invalid object pattern entry".into()))?;
                        let key = arr
                            .first()
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| OpError::Other("invalid object pattern key".into()))?;
                        let child_pattern = arr
                            .get(1)
                            .ok_or_else(|| OpError::Other("invalid object pattern entry".into()))?;
                        let child = obj.get(key).unwrap_or(&JsonValue::Null);
                        if !match_pattern(child_pattern, child, out)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                _ => {
                    return Err(OpError::Other(format!("unknown pattern type: {type_name}")));
                }
            }
        }
    }
    Ok(pattern == value)
}

// ── match ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Match;
impl Op for Match {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 2)?;
        let value = &args[0];
        for branch_arg in &args[1..] {
            let branch = branch_arg.as_array().ok_or(OpError::TypeMismatch(format!(
                "expected {}, got {}",
                "array [pattern, handler]", "non-array branch",
            )))?;
            if branch.len() < 2 {
                return Err(OpError::TypeMismatch(format!(
                    "expected {} args, got {}",
                    2,
                    branch.len(),
                )));
            }
            let pattern = &branch[0];
            let handler = &branch[1];
            let mut bindings = Vec::new();
            if match_pattern(pattern, value, &mut bindings)? {
                return call_with_bindings(ctx, handler, &bindings);
            }
        }
        Err(OpError::Other("no matching branch".into()))
    }
}

// NOTE: ADR 0040 gap — lazy if/then/else is not yet supported by the new
// non-generic Function trait. The eager If equivalent can be added when
// lazy evaluation support is available.

// ===========================================================================
// Application
// ===========================================================================

// ── apply ──────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Apply;
impl Op for Apply {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 2)?;
        let call_args = require_array(&args[1])?;
        apply_callable_args(ctx, &args[0], call_args)
    }
}

// ── tap ────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Tap;
impl Op for Tap {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        require_two_args(args, 2)?;
        let value = &args[1];
        let inner = value.clone();
        let _ = apply_callable(ctx, &args[0], &inner)?;
        Ok(value.clone())
    }
}

// ===========================================================================
// Collection iteration
// ===========================================================================

/// Require `(callable, array)` args — `args[0]` is the callback,
/// `args[1]` the array. The callback itself is validated lazily by the
/// `apply_callable*` helpers so arrow-values and `FunctionValue` JSON
/// both reach the right call path.
fn require_callable_and_array(
    args: &[serde_json::Value],
) -> Result<(&serde_json::Value, &[serde_json::Value]), OpError> {
    require_two_args(args, 2)?;
    let array = require_array(&args[1])?;
    Ok((&args[0], array))
}

// ── flat_map ───────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct FlatMap;
impl Op for FlatMap {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let (fn_arg, array) = require_callable_and_array(args)?;
        let mapped = map_callable_iter(ctx, fn_arg, array.iter())?;
        let mut results = Vec::new();
        for item in mapped {
            if let Some(sub) = item.as_array() {
                results.extend(sub.iter().cloned());
            } else {
                results.push(item);
            }
        }
        Ok(arr(&results))
    }
}

// ── find ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Find;
impl Op for Find {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let (fn_arg, array) = require_callable_and_array(args)?;
        for item in array {
            if is_truthy_callable(ctx, fn_arg, item)? {
                return Ok(item.clone());
            }
        }
        Ok(serde_json::Value::Null)
    }
}

// ── some / every ───────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct SomeFn;
impl Op for SomeFn {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let (fn_arg, array) = require_callable_and_array(args)?;
        for item in array {
            if is_truthy_callable(ctx, fn_arg, item)? {
                return Ok(b(true));
            }
        }
        Ok(b(false))
    }
}

#[derive(Clone, Copy)]
pub struct Every;
impl Op for Every {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let (fn_arg, array) = require_callable_and_array(args)?;
        for item in array {
            if !is_truthy_callable(ctx, fn_arg, item)? {
                return Ok(b(false));
            }
        }
        Ok(b(true))
    }
}

// ── group_by ───────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct GroupBy;
impl Op for GroupBy {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let (fn_arg, array) = require_callable_and_array(args)?;
        let mut groups: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
            std::collections::BTreeMap::new();
        for item in array {
            let key = apply_callable(ctx, fn_arg, item)?;
            let key_str = key_to_string(&key);
            groups.entry(key_str).or_default().push(item.clone());
        }
        let mut result = serde_json::Map::new();
        for (k, v) in groups {
            result.insert(k, serde_json::Value::Array(v));
        }
        Ok(serde_json::Value::Object(result))
    }
}

// ── partition ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Partition;
impl Op for Partition {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let (fn_arg, array) = require_callable_and_array(args)?;
        let mut matching = Vec::new();
        let mut rest = Vec::new();
        for item in array {
            if is_truthy_callable(ctx, fn_arg, item)? {
                matching.push(item.clone());
            } else {
                rest.push(item.clone());
            }
        }
        Ok(arr(&[
            serde_json::Value::Array(matching),
            serde_json::Value::Array(rest),
        ]))
    }
}

// ── count_by ───────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct CountBy;
impl Op for CountBy {
    fn call(
        &self,
        args: &[serde_json::Value],
        ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let (fn_arg, array) = require_callable_and_array(args)?;
        let mut counts: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
        for item in array {
            let key = apply_callable(ctx, fn_arg, item)?;
            let key_str = key_to_string(&key);
            *counts.entry(key_str).or_insert(0.0) += 1.0;
        }
        let mut result = serde_json::Map::new();
        for (k, v) in counts {
            result.insert(k, serde_json::json!(v));
        }
        Ok(serde_json::Value::Object(result))
    }
}

use handl_ops::OpRegistry;
use std::sync::Arc;

pub const COMBINATORS_OPS: &[&str] = &[
    "combinators.pipe",
    "combinators.fanout",
    "combinators.choice",
    "combinators.apply",
    "combinators.flat_map",
    "combinators.find",
    "combinators.some",
    "combinators.every",
    "combinators.group_by",
    "combinators.partition",
    "combinators.count_by",
    "combinators.tap",
    "combinators.match",
    "combinators.when",
    "combinators.unless",
    "combinators.converge",
];

pub fn combinators_op_registry() -> OpRegistry {
    let mut registry = OpRegistry::new();
    registry.register("combinators.pipe", Arc::new(Pipe));
    registry.register("combinators.fanout", Arc::new(Fanout));
    registry.register("combinators.choice", Arc::new(Choice));
    registry.register("combinators.apply", Arc::new(Apply));
    registry.register("combinators.flat_map", Arc::new(FlatMap));
    registry.register("combinators.find", Arc::new(Find));
    registry.register("combinators.some", Arc::new(SomeFn));
    registry.register("combinators.every", Arc::new(Every));
    registry.register("combinators.group_by", Arc::new(GroupBy));
    registry.register("combinators.partition", Arc::new(Partition));
    registry.register("combinators.count_by", Arc::new(CountBy));
    registry.register("combinators.tap", Arc::new(Tap));
    registry.register("combinators.match", Arc::new(Match));
    registry.register("combinators.when", Arc::new(When));
    registry.register("combinators.unless", Arc::new(Unless));
    registry.register("combinators.converge", Arc::new(Converge));
    registry
}
