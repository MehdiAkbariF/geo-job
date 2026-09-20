use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::register_handler,
        crate::handlers::auth::login_handler,
        crate::handlers::auth::me_handler,
        crate::handlers::discovery::search_opportunities_handler,
        crate::handlers::opportunity::create_opportunity_handler,
        crate::handlers::opportunity::publish_opportunity_handler,
        crate::handlers::opportunity::pause_opportunity_handler,
        crate::handlers::opportunity::close_opportunity_handler,
        crate::handlers::application::submit_application_handler,
        crate::handlers::application::change_application_status_handler,
        crate::handlers::saved::save_opportunity_handler,
        crate::handlers::saved::remove_saved_opportunity_handler,
        crate::handlers::saved::list_saved_opportunities_handler,
        crate::handlers::governance::report_opportunity_handler
    ),
    components(schemas()),
    tags(
        (name = "Auth", description = "Authentication & Identity management"),
        (name = "Opportunities", description = "Opportunity lifecycle management"),
        (name = "Discovery", description = "Search & Discovery orchestration"),
        (name = "Applications", description = "Candidate job applications"),
        (name = "Saved", description = "Saved opportunities & companies"),
        (name = "Governance", description = "Reports & platform governance")
    )
)]
pub struct ApiDoc;
