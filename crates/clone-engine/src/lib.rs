//! Clone planning only. Raw-device writes are intentionally not implemented.

use decode_core::{OperationKind, OperationPlan, SafetyError};

pub fn validate_clone(plan: &OperationPlan) -> Result<(), SafetyError> {
    if plan.kind != OperationKind::Clone {
        return Err(SafetyError::DestinationRequired);
    }
    plan.validate()
}
