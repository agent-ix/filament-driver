// SPDX-License-Identifier: AGPL-3.0-or-later
//! The canonical `TypeEnvironment` of a prepared program, built only from the
//! producer rows IR admitted in the selected closure.

use std::collections::BTreeMap;

use filament_contract_model::{
    CheckedNodeId, CheckedPackageV2, CheckedRational, SelectedMonitorTypeClosure,
};
use filament_exact::{Cancel, Integer, NodeKey, Rational, ValueType};
use filament_semantic_value::{
    declaration::{
        CompositeDeclaration, CompositeShape, FieldDeclaration, TypeEnvironment,
        TypeEnvironmentLimits, UnionMemberDeclaration,
    },
    quantity::UnitTable,
    unit::{DimensionNode, NominalDeclaration, UnitGraph, UnitNode},
};

use super::{
    closure::ClosureFacts, lowering::node_key, MonitorPreparationError as Error,
    MonitorPreparationLimits,
};

#[cfg(test)]
mod tests;

fn unsupported_source_type(package: &CheckedPackageV2, node: &CheckedNodeId) -> Error {
    let Some(kind) = package.node_kind(node) else {
        return Error::InvalidAdmittedScalar {
            node: Box::new(node.clone()),
        };
    };
    match crate::monitor_refusal::unsupported_at(package, node, &[], kind, None) {
        Ok(payload) => Error::Unsupported(Box::new(payload)),
        Err(error) => Error::Internal(error),
    }
}

fn source_record_fields(
    package: &CheckedPackageV2,
    record: &filament_contract_model::SelectedSourceRecordRow,
) -> Result<Vec<FieldDeclaration>, Error> {
    record
        .members
        .iter()
        .map(|member| {
            let value_type = member
                .value_type
                .clone()
                .ok_or_else(|| unsupported_source_type(package, &record.structural_node))?;
            Ok(FieldDeclaration::new(
                member.admitted_name.as_ref(),
                value_type,
                member.presence,
            ))
        })
        .collect()
}

pub(super) fn rational(value: &CheckedRational) -> Result<Rational, Error> {
    let part = |text: &str| text.parse::<Integer>().map_err(|_| Error::Rational);
    Rational::new(part(&value.numerator)?, part(&value.denominator)?).map_err(|_| Error::Rational)
}

fn units(
    closure: &SelectedMonitorTypeClosure<'_>,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<UnitTable, Error> {
    let declarations = closure.model_types();
    let mut dimensions = BTreeMap::new();
    for dimension in declarations.dimensions() {
        let mut terms = Vec::with_capacity(dimension.terms.len());
        for term in &dimension.terms {
            let exponent = term
                .exponent
                .parse::<Integer>()
                .map_err(|_| Error::Rational)?;
            terms.push((node_key(&term.dimension_node_id, limits, cancel)?, exponent));
        }
        let key = node_key(&dimension.structural_node, limits, cancel)?;
        let node = DimensionNode::checked(terms, NominalDeclaration::default())
            .map_err(|cause| Error::UnitGraph(Box::new(format!("{cause:?}"))))?;
        dimensions.insert(key, node);
    }
    let mut nodes: BTreeMap<NodeKey, UnitNode> = BTreeMap::new();
    let all = declarations
        .units()
        .iter()
        .map(|u| {
            (
                &u.structural_node,
                &u.dimension_node,
                &u.target_node,
                &u.scale,
                &u.offset,
            )
        })
        .chain(declarations.model_units().iter().map(|u| {
            (
                &u.structural_node,
                &u.dimension_node,
                &u.target_node,
                &u.scale,
                &u.offset,
            )
        }));
    for (node, dimension, target, scale, offset) in all {
        let target = match target {
            Some(target) => Some(node_key(target, limits, cancel)?),
            None => None,
        };
        let unit = UnitNode::checked(
            node_key(dimension, limits, cancel)?,
            target,
            rational(scale)?,
            rational(offset)?,
            NominalDeclaration::default(),
        )
        .map_err(|cause| Error::UnitGraph(Box::new(format!("{cause:?}"))))?;
        nodes.insert(node_key(node, limits, cancel)?, unit);
    }
    let graph = UnitGraph::from_checked_nodes(&dimensions, &nodes)
        .map_err(|cause| Error::UnitGraph(Box::new(format!("{cause:?}"))))?;
    Ok(UnitTable::declared(&graph))
}

/// Build the retained type environment: source records, tuples and unions from
/// the admitted producer rows, and the closure's unit table.
/// `package` is the closure's original admitted owner, used for typed source
/// diagnostics when a clock-bearing declaration has no runtime value form.
pub fn build_type_environment(
    package: &CheckedPackageV2,
    closure: &SelectedMonitorTypeClosure<'_>,
    facts: &ClosureFacts<'_>,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<TypeEnvironment, Error> {
    let name = |segments: &[Box<str>]| segments.join(".");
    let mut composites = Vec::new();
    for record in facts.source_records {
        let fields = source_record_fields(package, record)?;
        composites.push(CompositeDeclaration::new(
            record.record_key.clone(),
            name(&record.qualified_declaration),
            CompositeShape::Record(fields),
        ));
    }
    for tuple in facts.source_tuples {
        composites.push(CompositeDeclaration::new(
            tuple.type_handle.clone(),
            name(&tuple.qualified_declaration),
            CompositeShape::Tuple(
                tuple
                    .positions
                    .iter()
                    .map(|p| {
                        p.value_type
                            .clone()
                            .ok_or_else(|| unsupported_source_type(package, &tuple.structural_node))
                    })
                    .collect::<Result<Vec<ValueType>, Error>>()?,
            ),
        ));
    }
    for union in facts.source_unions {
        if union
            .members
            .iter()
            .any(|member| !member.clock_payload.is_empty())
        {
            return Err(unsupported_source_type(package, &union.structural_node));
        }
        let members = union
            .members
            .iter()
            .map(|member| {
                UnionMemberDeclaration::from_verified(member.member.clone(), member.payload.clone())
            })
            .collect();
        composites.push(CompositeDeclaration::new(
            union.type_handle.clone(),
            name(&union.qualified_declaration),
            CompositeShape::Union(members),
        ));
    }
    let table = units(closure, limits, cancel)?;
    let environment = TypeEnvironment::bounded_with_cancel(
        composites,
        [],
        // The driver's work allowance bounds the environment's admission work;
        // the ancestor-walk ceiling keeps the owner's default.
        TypeEnvironmentLimits {
            work_units: limits.work,
            ..TypeEnvironmentLimits::default()
        },
        cancel,
    )
    .map_err(|failure| Error::TypeEnvironment(Box::new(failure)))?;
    Ok(environment.with_units(table))
}
