use soroban_guard_checks::{
    Check, DeployAddressLostCheck, DeployArbitraryWasmCheck, InvokeNonexistentFuncCheck,
    InvokeResultStoredCheck, InvokeReturnCheck, UnvalidatedInvokeTargetCheck,
};
use std::fs;
use std::path::PathBuf;

fn read_fixture(fixture_name: &str) -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .expect("parent of checks is crates")
        .parent()
        .expect("parent of crates is repo root");
    let fixture_path = root
        .join("test-contracts")
        .join(fixture_name)
        .join("src")
        .join("lib.rs");
    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture at {:?}: {}", fixture_path, e))
}

#[test]
fn test_deploy_address_lost_fixture() {
    let check = DeployAddressLostCheck;
    let vuln_src = read_fixture("deploy-address-lost-vulnerable");
    let vuln_file: syn::File = syn::parse_str(&vuln_src).expect("parse vulnerable");
    let findings = check.run(&vuln_file, &vuln_src);
    assert!(
        !findings.is_empty(),
        "deploy-address-lost must fire on vulnerable fixture"
    );

    let safe_src = read_fixture("deploy-address-lost-safe");
    let safe_file: syn::File = syn::parse_str(&safe_src).expect("parse safe");
    let safe_findings = check.run(&safe_file, &safe_src);
    assert!(
        safe_findings.is_empty(),
        "deploy-address-lost must be silent on safe fixture"
    );
}

#[test]
fn test_deploy_arbitrary_wasm_fixture() {
    let check = DeployArbitraryWasmCheck;
    let vuln_src = read_fixture("deploy-arbitrary-wasm-vulnerable");
    let vuln_file: syn::File = syn::parse_str(&vuln_src).expect("parse vulnerable");
    let findings = check.run(&vuln_file, &vuln_src);
    assert!(
        !findings.is_empty(),
        "deploy-arbitrary-wasm must fire on vulnerable fixture"
    );

    let safe_src = read_fixture("deploy-arbitrary-wasm-safe");
    let safe_file: syn::File = syn::parse_str(&safe_src).expect("parse safe");
    let safe_findings = check.run(&safe_file, &safe_src);
    assert!(
        safe_findings.is_empty(),
        "deploy-arbitrary-wasm must be silent on safe fixture"
    );
}

#[test]
fn test_invoke_nonexistent_func_fixture() {
    let check = InvokeNonexistentFuncCheck;
    let vuln_src = read_fixture("invoke_nonexistent_func-vulnerable");
    let vuln_file: syn::File = syn::parse_str(&vuln_src).expect("parse vulnerable");
    let findings = check.run(&vuln_file, &vuln_src);
    assert!(
        !findings.is_empty(),
        "invoke-nonexistent-func must fire on vulnerable fixture"
    );

    let safe_src = read_fixture("invoke_nonexistent_func-safe");
    let safe_file: syn::File = syn::parse_str(&safe_src).expect("parse safe");
    let safe_findings = check.run(&safe_file, &safe_src);
    assert!(
        safe_findings.is_empty(),
        "invoke-nonexistent-func must be silent on safe fixture"
    );
}

#[test]
fn test_invoke_result_stored_fixture() {
    let check = InvokeResultStoredCheck;
    let vuln_src = read_fixture("invoke-result-stored-vulnerable");
    let vuln_file: syn::File = syn::parse_str(&vuln_src).expect("parse vulnerable");
    let findings = check.run(&vuln_file, &vuln_src);
    assert!(
        !findings.is_empty(),
        "invoke-result-stored must fire on vulnerable fixture"
    );

    let safe_src = read_fixture("invoke-result-stored-safe");
    let safe_file: syn::File = syn::parse_str(&safe_src).expect("parse safe");
    let safe_findings = check.run(&safe_file, &safe_src);
    assert!(
        safe_findings.is_empty(),
        "invoke-result-stored must be silent on safe fixture"
    );
}

#[test]
fn test_invoke_return_fixture() {
    let check = InvokeReturnCheck;
    let vuln_src = read_fixture("invoke-return-vulnerable");
    let vuln_file: syn::File = syn::parse_str(&vuln_src).expect("parse vulnerable");
    let findings = check.run(&vuln_file, &vuln_src);
    assert!(
        !findings.is_empty(),
        "invoke-return must fire on vulnerable fixture"
    );

    let safe_src = read_fixture("invoke-return-safe");
    let safe_file: syn::File = syn::parse_str(&safe_src).expect("parse safe");
    let safe_findings = check.run(&safe_file, &safe_src);
    assert!(
        safe_findings.is_empty(),
        "invoke-return must be silent on safe fixture"
    );
}

#[test]
fn test_unvalidated_invoke_target_fixture() {
    let check = UnvalidatedInvokeTargetCheck;
    let vuln_src = read_fixture("unvalidated-invoke-target-vulnerable");
    let vuln_file: syn::File = syn::parse_str(&vuln_src).expect("parse vulnerable");
    let findings = check.run(&vuln_file, &vuln_src);
    assert!(
        !findings.is_empty(),
        "unvalidated-invoke-target must fire on vulnerable fixture"
    );

    let safe_src = read_fixture("unvalidated-invoke-target-safe");
    let safe_file: syn::File = syn::parse_str(&safe_src).expect("parse safe");
    let safe_findings = check.run(&safe_file, &safe_src);
    assert!(
        safe_findings.is_empty(),
        "unvalidated-invoke-target must be silent on safe fixture"
    );
}
