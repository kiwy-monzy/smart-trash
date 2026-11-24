//! Payment RPC: invoices and payment recording. Amounts are integer cents.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rpc_common::authentication::{Caller, Role};
use rpc_common::validation;
use smarttrash_proto::payment::payment_service_server::PaymentService;
use smarttrash_proto::payment::{
    CreateInvoiceRequest, GetInvoiceRequest, Invoice, InvoiceStatus, RecordPaymentRequest,
};
use tonic::{Request, Response, Status};

pub struct PaymentRpc {
    invoices: Mutex<HashMap<String, Invoice>>,
    next_id: AtomicI64,
}

impl Default for PaymentRpc {
    fn default() -> Self {
        Self {
            invoices: Mutex::new(HashMap::new()),
            next_id: AtomicI64::new(0),
        }
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

#[tonic::async_trait]
impl PaymentService for PaymentRpc {
    async fn create_invoice(
        &self,
        request: Request<CreateInvoiceRequest>,
    ) -> Result<Response<Invoice>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("invoice.create")?;
        let data = request.into_inner();

        validation::identifier("customer_id", &data.customer_id)?;
        if data.amount_cents <= 0 || data.amount_cents > 100_000_000_000 {
            return Err(rpc_common::invalid_argument(
                "amount_cents must be within [1, 100000000000]",
            ));
        }
        let currency = if data.currency.trim().is_empty() {
            "TZS".to_string()
        } else {
            validation::text("currency", &data.currency, 8)?;
            data.currency
        };

        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let invoice = Invoice {
            invoice_id: format!("INV-{id:05}"),
            customer_id: data.customer_id,
            pickup_id: data.pickup_id,
            amount_cents: data.amount_cents,
            currency,
            status: InvoiceStatus::Issued as i32,
            issued_at_unix: unix_now(),
            paid_at_unix: 0,
        };

        rpc_common::lock(&self.invoices).insert(invoice.invoice_id.clone(), invoice.clone());

        tracing::info!(
            invoice_id = %invoice.invoice_id,
            amount_cents = invoice.amount_cents,
            caller = %caller.user_id,
            "invoice issued"
        );
        Ok(Response::new(invoice))
    }

    async fn get_invoice(
        &self,
        request: Request<GetInvoiceRequest>,
    ) -> Result<Response<Invoice>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("invoice.get")?;
        let id = request.into_inner().invoice_id;
        validation::identifier("invoice_id", &id)?;

        let invoice = rpc_common::lock(&self.invoices)
            .get(&id)
            .cloned()
            .ok_or_else(|| rpc_common::not_found("Invoice", &id))?;

        if caller.role != Role::Council && invoice.customer_id != caller.user_id {
            return Err(Status::permission_denied(
                "this invoice belongs to another customer",
            ));
        }
        Ok(Response::new(invoice))
    }

    async fn record_payment(
        &self,
        request: Request<RecordPaymentRequest>,
    ) -> Result<Response<Invoice>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("invoice.pay")?;
        let data = request.into_inner();

        validation::identifier("invoice_id", &data.invoice_id)?;
        validation::text("payment_reference", &data.payment_reference, 64)?;

        let mut invoices = rpc_common::lock(&self.invoices);
        let invoice = invoices
            .get_mut(&data.invoice_id)
            .ok_or_else(|| rpc_common::not_found("Invoice", &data.invoice_id))?;

        if caller.role != Role::Council && invoice.customer_id != caller.user_id {
            return Err(Status::permission_denied(
                "this invoice belongs to another customer",
            ));
        }

        if invoice.status() != InvoiceStatus::Issued && invoice.status() != InvoiceStatus::Overdue {
            return Err(rpc_common::failed_precondition(format!(
                "invoice '{}' is {:?}, expected ISSUED or OVERDUE",
                data.invoice_id,
                invoice.status()
            )));
        }
        if data.paid_amount_cents != invoice.amount_cents {
            return Err(rpc_common::failed_precondition(format!(
                "paid amount {} does not match invoice amount {}",
                data.paid_amount_cents, invoice.amount_cents
            )));
        }

        invoice.status = InvoiceStatus::Paid as i32;
        invoice.paid_at_unix = unix_now();

        tracing::info!(
            invoice_id = %invoice.invoice_id,
            reference = %data.payment_reference,
            caller = %caller.user_id,
            "payment recorded"
        );
        Ok(Response::new(invoice.clone()))
    }
}
