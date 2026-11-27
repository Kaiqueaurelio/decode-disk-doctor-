//! Structured report model. PDF rendering will be added after report schema review.

use diagnostic_engine::DiagnosticSnapshot;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceReport {
    pub id: Uuid,
    pub technician_name: Option<String>,
    pub customer_reference: Option<String>,
    pub equipment_reference: Option<String>,
    pub summary: String,
    pub diagnostics: Vec<DiagnosticSnapshot>,
    pub limitations: Vec<String>,
}
