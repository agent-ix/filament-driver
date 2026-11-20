// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real help/version behavior checked against the product-owned option contract.

// This dependency-free product module explicitly supports integration by path.
// Its other public APIs belong to the product; this target uses table and scan.
mod help_cases;
#[allow(dead_code)]
#[path = "../../fsl-cli/src/options.rs"]
mod production_options;
