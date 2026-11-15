// SPDX-License-Identifier: AGPL-3.0-or-later
//! Closed direct scalar mappings; source admission and accounting stay with the caller.

use filament_contract_model::{ApplicationOperator, CheckedOperationRead};
use filament_contract_runtime::monitor::{ExpressionId, Operation};
use filament_exact::{OrderingOperator, ValueType};

/// Map only the exact admitted plain operation/shape declared by FR-017.
/// `None` asks the caller to retain the original located Unsupported payload;
/// it never grants permission to evaluate or rewrite the source operation.
pub(super) fn map_application(
    operator: &ApplicationOperator,
    source: CheckedOperationRead<'_>,
    arguments: &[ExpressionId],
    operands: &[ValueType],
    result: &ValueType,
) -> Option<Operation> {
    if !accepts_header(operator, source, arguments.len()) {
        return None;
    }
    match (operator, arguments, operands, result) {
        (
            ApplicationOperator::Binary,
            [left, right],
            [ValueType::Integer, ValueType::Integer],
            ValueType::Integer,
        ) => match source.identity {
            "filament.op.integer.add" => Some(Operation::IntegerAdd {
                left: *left,
                right: *right,
            }),
            "filament.op.integer.sub" => Some(Operation::IntegerSubtract {
                left: *left,
                right: *right,
            }),
            "filament.op.integer.mul" => Some(Operation::IntegerMultiply {
                left: *left,
                right: *right,
            }),
            _ => None,
        },
        (ApplicationOperator::Unary, [value], [ValueType::Integer], ValueType::Integer)
            if source.identity == "filament.op.integer.negate" =>
        {
            Some(Operation::IntegerNegate { value: *value })
        }
        (
            ApplicationOperator::Binary,
            [left, right],
            [ValueType::Integer, ValueType::Integer],
            ValueType::Boolean,
        ) => {
            let operator = match source.identity {
                "filament.op.integer.lt" => OrderingOperator::Less,
                "filament.op.integer.le" => OrderingOperator::LessOrEqual,
                "filament.op.integer.gt" => OrderingOperator::Greater,
                "filament.op.integer.ge" => OrderingOperator::GreaterOrEqual,
                _ => return None,
            };
            Some(Operation::IntegerOrder {
                operator,
                left: *left,
                right: *right,
            })
        }
        (ApplicationOperator::Unary, [value], [ValueType::Boolean], ValueType::Boolean)
            if source.identity == "filament.op.boolean.not" =>
        {
            Some(Operation::BooleanNot { value: *value })
        }
        (
            ApplicationOperator::Binary,
            [left, right],
            [ValueType::Boolean, ValueType::Boolean],
            ValueType::Boolean,
        ) => match source.identity {
            "filament.op.boolean.and" => Some(Operation::BooleanAnd {
                left: *left,
                right: *right,
            }),
            "filament.op.boolean.or" => Some(Operation::BooleanOr {
                left: *left,
                right: *right,
            }),
            _ => None,
        },
        _ => None,
    }
}

/// Reject an excluded operation at its own locus before visiting its operands.
pub(super) fn accepts_header(
    operator: &ApplicationOperator,
    source: CheckedOperationRead<'_>,
    arity: usize,
) -> bool {
    if source.mode.is_some()
        || source.member.is_some()
        || source.execution_point.is_some()
        || !source.laws.is_empty()
        || !source.constraints.is_empty()
        || !source.leaves.is_empty()
    {
        return false;
    }
    matches!(
        (operator, arity, source.identity),
        (
            ApplicationOperator::Unary,
            1,
            "filament.op.integer.negate" | "filament.op.boolean.not"
        ) | (
            ApplicationOperator::Binary,
            2,
            "filament.op.integer.add"
                | "filament.op.integer.sub"
                | "filament.op.integer.mul"
                | "filament.op.integer.lt"
                | "filament.op.integer.le"
                | "filament.op.integer.gt"
                | "filament.op.integer.ge"
                | "filament.op.boolean.and"
                | "filament.op.boolean.or"
        )
    )
}

/// Only variants constructed by this closed capability can be retained at an
/// original transparent-reference location. Future allocating variants must
/// not enter this copy through a generic Operation clone.
pub(super) fn copy_scalar_operation(source: &Operation) -> Option<Operation> {
    Some(match source {
        Operation::Argument { function, position } => Operation::Argument {
            function: *function,
            position: *position,
        },
        Operation::IntegerAdd { left, right } => Operation::IntegerAdd {
            left: *left,
            right: *right,
        },
        Operation::IntegerSubtract { left, right } => Operation::IntegerSubtract {
            left: *left,
            right: *right,
        },
        Operation::IntegerMultiply { left, right } => Operation::IntegerMultiply {
            left: *left,
            right: *right,
        },
        Operation::IntegerNegate { value } => Operation::IntegerNegate { value: *value },
        Operation::IntegerOrder {
            operator,
            left,
            right,
        } => Operation::IntegerOrder {
            operator: *operator,
            left: *left,
            right: *right,
        },
        Operation::BooleanNot { value } => Operation::BooleanNot { value: *value },
        Operation::BooleanAnd { left, right } => Operation::BooleanAnd {
            left: *left,
            right: *right,
        },
        Operation::BooleanOr { left, right } => Operation::BooleanOr {
            left: *left,
            right: *right,
        },
        _ => return None,
    })
}
