use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::finance::{CreateInvoiceCommand, InvoiceDto, TariffDto, TransactionDto, WalletDto};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/wallet",
    responses((status = 200, description = "Company wallet details and balances", body = WalletDto)),
    tag = "Finance"
)]
pub async fn get_company_wallet_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(company_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let wallet = state.finance_use_cases.get_company_wallet(auth.user_id, company_id).await?;
    Ok(Json(wallet))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/tariffs",
    responses((status = 200, description = "List all platform tariffs and credit packages", body = Vec<TariffDto>)),
    tag = "Finance"
)]
pub async fn list_tariffs_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.finance_use_cases.list_tariffs().await?;
    Ok(Json(list))
}

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/finance/invoices",
    request_body = CreateInvoiceCommand,
    responses((status = 201, description = "Invoice created for package", body = InvoiceDto)),
    tag = "Finance"
)]
pub async fn create_invoice_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    Json(cmd): Json<CreateInvoiceCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let invoice = state.finance_use_cases.create_package_invoice(auth.user_id, company_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(invoice)))
}

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/finance/invoices/{invoice_id}/pay-mock",
    responses((status = 200, description = "Invoice paid via sandbox gateway and wallet charged", body = WalletDto)),
    tag = "Finance"
)]
pub async fn mock_pay_invoice_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((company_id, invoice_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let wallet = state.finance_use_cases.mock_pay_invoice(auth.user_id, company_id, invoice_id).await?;
    Ok(Json(wallet))
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/finance/transactions",
    responses((status = 200, description = "List wallet transactions ledger", body = Vec<TransactionDto>)),
    tag = "Finance"
)]
pub async fn list_wallet_transactions_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(company_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.finance_use_cases.list_transactions(auth.user_id, company_id).await?;
    Ok(Json(list))
}

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/talents/{candidate_id}/unlock",
    responses((status = 200, description = "Candidate contact unlocked via wallet deduction")),
    tag = "Finance"
)]
pub async fn unlock_talent_contact_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((company_id, candidate_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    state.finance_use_cases.unlock_candidate_contact(auth.user_id, company_id, candidate_id).await?;
    Ok(StatusCode::OK)
}