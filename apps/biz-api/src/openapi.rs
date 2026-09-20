use utoipa::OpenApi;

use crate::handlers::auth::RefreshTokenRequest;
use crate::handlers::taxonomy::SkillQuery;
use biz_application::application::{
    ApplicationDossierDto, ApplicationDto, ChangeApplicationStatusCommand,
    SubmitApplicationCommand,
};
use biz_application::candidate::{
    AddExperienceCommand, CandidatePreferencesDto, CandidateProfileDto, ExperienceDto,
    SetSkillsCommand, TrackedApplicationDto, UpdateProfileCommand,
};
use biz_application::company::{
    AddCompanyLocationCommand, AddMemberCommand, CompanyDto, CompanyMemberDto,
    CreateCompanyCommand, UpdateCompanyCommand,
};
use biz_application::governance::{
    CompanyVerificationDto, CreateReportCommand, ReviewVerificationCommand,
    SubmitVerificationCommand,
};
use biz_application::identity::{AuthResponseDto, LoginCommand, RegisterCommand};
use biz_application::opportunity::CreateOpportunityCommand;
use biz_application::saved::SaveSearchCommand;

use biz_domain::application::{Application, ApplicationStatus};
use biz_domain::discovery::{CompanySummary, OpportunitySearchResult, SearchPageResult};
use biz_domain::governance::{Report, VerificationStatus};
use biz_domain::opportunity::{Opportunity, OpportunityStatus, OpportunityType, RemoteScope, Salary, WorkplaceType};
use biz_domain::taxonomy::{Category, ExperienceLevel, Industry, Occupation, Skill};

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

        // Companies Management & Showcase
        crate::handlers::company::create_company_handler,
        crate::handlers::company::update_company_handler,
        crate::handlers::company::get_company_handler,
        crate::handlers::company::add_company_location_handler,
        crate::handlers::company::list_members_handler,
        crate::handlers::company::add_member_handler,
        crate::handlers::company::list_company_opportunities_handler,
        crate::handlers::company::list_public_company_opportunities_handler,
        crate::handlers::company::list_opportunity_applicants_handler,

        // Discovery & Map Bridge
        crate::handlers::discovery::search_opportunities_handler,
        crate::handlers::discovery::get_opportunities_by_location_handler,
        crate::handlers::opportunity::get_opportunity_handler,
        crate::handlers::opportunity::track_opportunity_click_handler,

        // Opportunities Lifecycle
        crate::handlers::opportunity::create_opportunity_handler,
        crate::handlers::opportunity::publish_opportunity_handler,
        crate::handlers::opportunity::pause_opportunity_handler,
        crate::handlers::opportunity::resume_opportunity_handler,
        crate::handlers::opportunity::close_opportunity_handler,

        // Applications & Employer ATS Dossier
        crate::handlers::application::submit_application_handler,
        crate::handlers::application::get_application_dossier_handler,
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

        // Taxonomies
        crate::handlers::taxonomy::list_categories_handler,
        crate::handlers::taxonomy::list_industries_handler,
        crate::handlers::taxonomy::list_occupations_handler,
        crate::handlers::taxonomy::resolve_skill_handler,

        // Governance & Trust
        crate::handlers::governance::report_opportunity_handler,
        crate::handlers::governance::submit_verification_handler,
        crate::handlers::governance::list_pending_verifications_handler,
        crate::handlers::governance::review_verification_handler,
        crate::handlers::governance::list_reports_handler
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
            CreateCompanyCommand,
            UpdateCompanyCommand,
            CompanyDto,
            AddCompanyLocationCommand,
            AddMemberCommand,
            CompanyMemberDto,
            CreateOpportunityCommand,
            SubmitApplicationCommand,
            ChangeApplicationStatusCommand,
            ApplicationDto,
            Application,
            ApplicationStatus,
            ApplicationDossierDto,
            SaveSearchCommand,
            CreateReportCommand,
            SubmitVerificationCommand,
            ReviewVerificationCommand,
            CompanyVerificationDto,
            VerificationStatus,
            Report,
            Opportunity,
            OpportunityType,
            WorkplaceType,
            RemoteScope,
            ExperienceLevel,
            Salary,
            OpportunityStatus,
            Category,
            Industry,
            Occupation,
            Skill,
            CompanySummary,
            OpportunitySearchResult,
            SearchPageResult
        )
    ),
    tags(
        (name = "Auth", description = "Authentication, tokens & sessions"),
        (name = "Candidate Profile", description = "Talent profile, experiences, skills & application tracking"),
        (name = "Companies", description = "Company profile, branch locations & team membership"),
        (name = "Employer ATS", description = "Applicant tracking system, dossiers & opportunity pipeline"),
        (name = "Opportunities", description = "Opportunity lifecycle management (Draft, Publish, Pause, Close)"),
        (name = "Discovery", description = "Search & Discovery orchestration and Map Pin bridging"),
        (name = "Taxonomies", description = "Standard industries, job categories, occupations & skills autocomplete"),
        (name = "Applications", description = "Candidate job applications pipeline"),
        (name = "Saved", description = "Saved opportunities, companies & searches"),
        (name = "Governance & Trust", description = "Platform trust, company legal verifications & moderation reports")
    )
)]
pub struct ApiDoc;