use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(),
    tags(
        (name = "Auth", description = "Authentication & Identity management"),
        (name = "Opportunities", description = "Opportunity lifecycle management"),
        (name = "Discovery", description = "Search & Discovery orchestration"),
        (name = "Applications", description = "Candidate job applications")
    )
)]
pub struct ApiDoc;