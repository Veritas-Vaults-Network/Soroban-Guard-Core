#!/bin/bash
set -e

# Issue #66 - Assert For Auth
git checkout issue-66-assert-for-auth
git add -A
git commit -m "Issue #66: Add assert-for-auth check

- Implement AssertForAuthCheck to detect assert! macro used for access control
- Add assert-for-auth-vulnerable test contract
- Add assert-for-auth-safe test contract
- Register check in default_checks()"
git push origin issue-66-assert-for-auth

# Issue #68 - Authorize As Contract  
git checkout main
git checkout issue-68-authorize-as-contract
git add -A
git commit -m "Issue #68: Add authorize-as-contract check

- Implement AuthorizeAsContractCheck to detect authorize_as_current_contract without prior require_auth
- Add authorize-as-contract-vulnerable test contract
- Add authorize-as-contract-safe test contract
- Register check in default_checks()"
git push origin issue-68-authorize-as-contract

# Issue #69 - Map Key Explosion
git checkout main
git checkout issue-69-map-key-explosion
git add -A
git commit -m "Issue #69: Add map-key-explosion check

- Implement MapKeyExplosionCheck to detect Map with excessive distinct string literal keys
- Add map-key-explosion-vulnerable test contract
- Add map-key-explosion-safe test contract
- Register check in default_checks()"
git push origin issue-69-map-key-explosion

echo "All branches committed and pushed successfully!"
