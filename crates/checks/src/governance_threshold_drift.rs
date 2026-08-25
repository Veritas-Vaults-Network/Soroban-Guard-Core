//! Detects governance-threshold drift: two `#[contractimpl]` methods that both
//! gate on a counter-like variable (vote / signer count) but use different
//! integer thresholds for what is meant to be the same quorum concept.
//!
//! This is the governance analogue of `scale-factor-drift`: each call site is
//! internally consistent, but a mismatch between independent call sites
//! silently weakens (or strengthens) the governance gate.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ExprBinary, ExprLit, File, Item, Lit};

const CHECK_NAME: &str = "governance-threshold-drift";

/// Flags governance functions that gate on the same counter variable but use
/// different threshold values.
pub struct GovernanceThresholdDriftCheck;

impl Check for GovernanceThresholdDriftCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let consts = collect_file_consts(file);
        let mut sites: Vec<ThresholdSite> = Vec::new();

        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            if !is_governance_fn(&fn_name) {
                continue;
            }
            let mut scanner = ThresholdScanner {
                fn_name: fn_name.clone(),
                _fn_line: method.sig.fn_token.span().start().line,
                consts: &consts,
                sites: Vec::new(),
            };
            scanner.visit_block(&method.block);
            sites.extend(scanner.sites);
        }

        let mut by_counter: HashMap<String, Vec<&ThresholdSite>> = HashMap::new();
        for site in &sites {
            by_counter
                .entry(site.counter.clone())
                .or_default()
                .push(site);
        }

        let mut out = Vec::new();
        for (counter, counter_sites) in &by_counter {
            let mut thresholds: Vec<&str> =
                counter_sites.iter().map(|s| s.threshold.as_str()).collect();
            thresholds.sort();
            thresholds.dedup();
            if thresholds.len() < 2 {
                continue;
            }

            let mut seen_thresholds: Vec<&str> = Vec::new();
            for site in counter_sites {
                if seen_thresholds.contains(&site.threshold.as_str()) {
                    continue;
                }
                seen_thresholds.push(&site.threshold);

                let other_sites: Vec<&&ThresholdSite> = counter_sites
                    .iter()
                    .filter(|s| s.threshold != site.threshold)
                    .collect();
                let other_summary = other_sites
                    .iter()
                    .map(|s| {
                        format!(
                            "`{}` in `{}` (line {}) uses {}",
                            s.op, s.fn_name, s.line, s.threshold
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");

                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line: site.line,
                    function_name: site.fn_name.clone(),
                    description: format!(
                        "Governance function `{}` gates on `{}` with threshold {}, \
                         but {} uses a different threshold for the same counter variable. \
                         Every governance function that gates on the same quorum concept \
                         must agree on the threshold value, or the governance check can \
                         be bypassed.",
                        site.fn_name, counter, site.threshold, other_summary
                    ),
                });
            }
        }
        out
    }
}

struct ThresholdSite {
    counter: String,
    threshold: String,
    op: String,
    line: usize,
    fn_name: String,
}

struct ThresholdScanner<'a> {
    fn_name: String,
    _fn_line: usize,
    consts: &'a HashMap<String, String>,
    sites: Vec<ThresholdSite>,
}

impl<'ast> Visit<'ast> for ThresholdScanner<'_> {
    fn visit_expr_binary(&mut self, i: &'ast ExprBinary) {
        if let Some(site) = try_extract_threshold_site(i, &self.fn_name, self.consts) {
            self.sites.push(site);
        }
        visit::visit_expr_binary(self, i);
    }
}

fn try_extract_threshold_site(
    bin: &ExprBinary,
    fn_name: &str,
    consts: &HashMap<String, String>,
) -> Option<ThresholdSite> {
    let op = bin_op_name(&bin.op)?;

    let left_counter = extract_counter_name(&bin.left);
    let right_threshold = extract_threshold_value(&bin.right, consts);
    if let (Some(counter), Some(threshold)) = (left_counter, right_threshold) {
        return Some(ThresholdSite {
            counter,
            threshold,
            op: format!("{}{}", op, "="),
            line: bin.span().start().line,
            fn_name: fn_name.to_string(),
        });
    }

    let right_counter = extract_counter_name(&bin.right);
    let left_threshold = extract_threshold_value(&bin.left, consts);
    if let (Some(counter), Some(threshold)) = (right_counter, left_threshold) {
        return Some(ThresholdSite {
            counter,
            threshold,
            op: format!("{}{}", op, "="),
            line: bin.span().start().line,
            fn_name: fn_name.to_string(),
        });
    }

    None
}

fn bin_op_name(op: &BinOp) -> Option<&'static str> {
    match op {
        BinOp::Ge(_) => Some(">"),
        BinOp::Gt(_) => Some(">"),
        BinOp::Le(_) => Some("<"),
        BinOp::Lt(_) => Some("<"),
        BinOp::Eq(_) => Some("="),
        _ => None,
    }
}

fn extract_counter_name(expr: &Expr) -> Option<String> {
    let name = expr_path_ident(expr)?;
    if is_counter_name(&name) {
        Some(name.to_lowercase())
    } else {
        None
    }
}

fn extract_threshold_value(expr: &Expr, consts: &HashMap<String, String>) -> Option<String> {
    if let Some(lit) = int_literal_str(expr) {
        return Some(lit);
    }
    if let Some(name) = expr_path_ident(expr) {
        if let Some(val) = consts.get(&name) {
            return Some(val.clone());
        }
    }
    None
}

fn expr_path_ident(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(p) if p.path.segments.len() == 1 => Some(p.path.segments[0].ident.to_string()),
        _ => None,
    }
}

fn int_literal_str(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => Some(i.base10_digits().to_string()),
        _ => None,
    }
}

fn is_counter_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("vote")
        || lower.contains("signer")
        || lower.contains("count")
        || lower.contains("yes")
        || lower.contains("approval")
        || lower.contains("support")
        || lower.contains("nay")
        || lower.contains("voter")
}

fn is_governance_fn(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("propose")
        || lower.contains("vote")
        || lower.contains("execute")
        || lower.contains("quorum")
        || lower.contains("threshold")
        || lower.contains("approve")
        || lower.contains("ratify")
        || lower.contains("finalize")
        || lower.contains("submit")
}

fn collect_file_consts(file: &File) -> HashMap<String, String> {
    let mut consts = HashMap::new();
    for item in &file.items {
        if let Item::Const(c) = item {
            let name = c.ident.to_string();
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(lit), ..
            }) = &*c.expr
            {
                consts.insert(name, lit.base10_digits().to_string());
            }
        }
    }
    consts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run(src: &str) -> Result<Vec<Finding>, syn::Error> {
        let file = parse_file(src)?;
        Ok(GovernanceThresholdDriftCheck.run(&file, src))
    }

    #[test]
    fn flags_different_thresholds_in_propose_and_execute() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Address};

pub struct C;

#[contractimpl]
impl C {
    pub fn propose(env: Env, signers: u32) -> bool {
        if signers >= 3 {
            true
        } else {
            false
        }
    }

    pub fn execute(env: Env, signers: u32) -> bool {
        if signers >= 2 {
            true
        } else {
            false
        }
    }
}
"#)?;
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|h| h.check_name == CHECK_NAME));
        assert!(hits.iter().all(|h| h.severity == Severity::High));
        assert!(hits.iter().any(|h| h.function_name == "propose"));
        assert!(hits.iter().any(|h| h.function_name == "execute"));
        Ok(())
    }

    #[test]
    fn passes_when_same_named_const_used() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Address};

const QUORUM: u32 = 3;

pub struct C;

#[contractimpl]
impl C {
    pub fn propose(env: Env, signers: u32) -> bool {
        if signers >= QUORUM {
            true
        } else {
            false
        }
    }

    pub fn execute(env: Env, signers: u32) -> bool {
        if signers >= QUORUM {
            true
        } else {
            false
        }
    }
}
"#)?;
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_when_same_literal_used() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Address};

pub struct C;

#[contractimpl]
impl C {
    pub fn propose(env: Env, signers: u32) -> bool {
        if signers >= 3 {
            true
        } else {
            false
        }
    }

    pub fn execute(env: Env, signers: u32) -> bool {
        if signers >= 3 {
            true
        } else {
            false
        }
    }
}
"#)?;
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn ignores_non_governance_functions() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Address};

pub struct C;

#[contractimpl]
impl C {
    pub fn deposit(env: Env, signers: u32) -> bool {
        if signers >= 3 {
            true
        } else {
            false
        }
    }

    pub fn withdraw(env: Env, signers: u32) -> bool {
        if signers >= 2 {
            true
        } else {
            false
        }
    }
}
"#)?;
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn flags_three_functions_with_mixed_thresholds() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Address};

pub struct C;

#[contractimpl]
impl C {
    pub fn propose(env: Env, signers: u32) -> bool {
        if signers >= 3 {
            true
        } else {
            false
        }
    }

    pub fn vote(env: Env, signers: u32) -> bool {
        if signers >= 3 {
            true
        } else {
            false
        }
    }

    pub fn execute(env: Env, signers: u32) -> bool {
        if signers >= 2 {
            true
        } else {
            false
        }
    }
}
"#)?;
        assert!(!hits.is_empty());
        assert!(hits.iter().any(|h| h.function_name == "execute"));
        Ok(())
    }
}
