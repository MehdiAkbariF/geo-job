use utoipa::OpenApi;

use crate::handlers::auth::RefreshTokenRequest;
use biz_application::application::{ApplicationDto, ChangeApplicationStatusCommand, SubmitApplicationCommand};
use biz_application::candidate::{
    AddExperienceCommand, CandidatePreferencesDto, CandidateProfileDto, ExperienceDto,
    SetSkillsCommand, TrackedApplicationDto, UpdateProfileCommand,
};
use biz_application::governance::CreateReportCommand;
use biz_application::identity::{AuthResponseDto, LoginCommand, RegisterCommand};
use biz_application::opportunity::CreateOpportunityCommand;
use biz_application::saved::SaveSearchCommand;

use biz_domain::opportunity::{Opportunity, OpportunityStatus, OpportunityType, RemoteScope, Salary, WorkplaceType};
use biz_domain::taxonomy::ExperienceLevel;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth
        crate::handlers::auth::register_handler,
        crate::handlers::auth::login_handler,
        crate::handlers::auth::refresh_handler,
        crate::handlers::auth::logout_handler,
        crate::handlers::auth::me_handler,

        // Candidate Profile & Actions
        crate::handlers::candidate::get_my_profile_handler,
        crate::handlers::candidate::update_my_profile_handler,
        crate::handlers::candidate::add_experience_handler,
        crate::handlers::candidate::delete_experience_handler,
        crate::handlers::candidate::set_skills_handler,
        crate::handlers::candidate::get_preferences_handler,
        crate::handlers::candidate::set_preferences_handler,
        crate::handlers::candidate::list_my_applications_handler,

        // Discovery
        crate::handlers::discovery::search_opportunities_handler,

        // Opportunities Lifecycle
        crate::handlers::opportunity::create_opportunity_handler,
        crate::handlers::opportunity::publish_opportunity_handler,
        crate::handlers::opportunity::pause_opportunity_handler,
        crate::handlers::opportunity::close_opportunity_handler,

        // Applications
        crate::handlers::application::submit_application_handler,
        crate::handlers::application::change_application_status_handler,

        // Saved Items
        crate::handlers::saved::save_opportunity_handler,
        crate::handlers::saved::remove_saved_opportunity_handler,
        crate::handlers::saved::list_saved_opportunities_handler,
        crate::handlers::saved::save_company_handler,
        crate::handlers::saved::remove_saved_company_handler,
        crate::handlers::saved::list_saved_companies_handler,
        crate::handlers::saved::save_search_handler,
        crate::handlers::saved::list_saved_searches_handler,

        // Governance
        crate::handlers::governance::report_opportunity_handler
    ),
    components(
        schemas(
            RegisterCommand,
            LoginCommand,
            RefreshTokenRequest,
            AuthResponseDto,
            UpdateProfileCommand,
            AddExperienceCommand,
            ExperienceDto,
            CandidateProfileDto,
            SetSkillsCommand,
            CandidatePreferencesDto,
            TrackedApplicationDto,
            CreateOpportunityCommand,
            SubmitApplicationCommand,
            ChangeApplicationStatusCommand,
            ApplicationDto,
            SaveSearchCommand,
            CreateReportCommand,
            Opportunity,
            OpportunityType,
            WorkplaceType,
            RemoteScope,
            ExperienceLevel,
            Salary,
            OpportunityStatus
        )
    ),
    tags(
        (name = "Auth", description = "Authentication, tokens & sessions"),
        (name = "Candidate Profile", description = "Talent profile, experiences, skills & application tracking"),
        (name = "Opportunities", description = "Opportunity lifecycle management"),
        (name = "Discovery", description = "Search & Discovery orchestration"),
        (name = "Applications", description = "Candidate job applications pipeline"),
        (name = "Saved", description = "Saved opportunities, companies & searches"),
        (name = "Governance", description = "Reports & platform governance")
    )
)]
pub struct ApiDoc;