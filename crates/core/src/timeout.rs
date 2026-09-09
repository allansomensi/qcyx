//! Timeout budgets for every crossing into the platform BLE stack.

use crate::error::CoreError;
use std::future::Future;
use std::time::Duration;

/// Budgets, sized generously: they exist to bound a hang, not to police latency.
pub mod budget {
    use std::time::Duration;

    /// Reads, writes, subscriptions, liveness checks.
    pub const GATT_OP: Duration = Duration::from_secs(5);

    /// A single `connect()` attempt, before retries.
    pub const CONNECT: Duration = Duration::from_secs(12);

    /// `discover_services()` plus the characteristic walk.
    pub const DISCOVERY: Duration = Duration::from_secs(8);

    /// Adapter enumeration, scan start/stop, peripheral listing.
    pub const SCAN_CONTROL: Duration = Duration::from_secs(5);
}

/// Runs `fut` under `budget`, mapping expiry to [`CoreError::OperationTimeout`].
pub async fn guard<F, T>(budget: Duration, label: &'static str, fut: F) -> Result<T, CoreError>
where
    F: Future<Output = Result<T, CoreError>>,
{
    match tokio::time::timeout(budget, fut).await {
        Ok(result) => result,
        Err(_elapsed) => {
            tracing::warn!(
                operation = label,
                budget_ms = budget.as_millis() as u64,
                "BLE operation timed out"
            );
            Err(CoreError::OperationTimeout(label))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn passes_through_a_prompt_result() {
        let value = guard(budget::GATT_OP, "test", async { Ok(7u8) })
            .await
            .unwrap();
        assert_eq!(value, 7);
    }

    #[tokio::test]
    async fn maps_expiry_to_operation_timeout() {
        let result: Result<(), CoreError> = guard(Duration::from_millis(10), "stuck", async {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Ok(())
        })
        .await;

        assert!(matches!(result, Err(CoreError::OperationTimeout("stuck"))));
    }
}
