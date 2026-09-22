use biz_application::application::ApplicationUseCases;
use biz_application::candidate::CandidateUseCases;
use biz_application::company::CompanyUseCases;
use biz_application::discovery::DiscoveryUseCases;
use biz_application::governance::GovernanceUseCases;
use biz_application::opportunity::OpportunityUseCases;
use biz_application::saved::SavedUseCases;
use biz_application::taxonomy::TaxonomyUseCases;
use biz_storage::{
    ApplicationRepository, CandidateRepository, CompanyRepository, DiscoveryRepository,
    GovernanceRepository, OpportunityRepository, SavedRepository, TaxonomyRepository,
    TokenRepository, TokenService, UserRepository,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub token_service: TokenService,
    pub auth_use_cases: biz_application::identity::AuthUseCases,
    pub opp_use_cases: OpportunityUseCases,
    pub discovery_use_cases: DiscoveryUseCases,
    pub candidate_use_cases: CandidateUseCases,
    pub app_use_cases: ApplicationUseCases,
    pub saved_use_cases: SavedUseCases,
    pub gov_use_cases: GovernanceUseCases,
    pub company_use_cases: CompanyUseCases,
    pub taxonomy_use_cases: TaxonomyUseCases,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = UserRepository::new(pool.clone());
        let token_repo = TokenRepository::new(pool.clone());
        let company_repo = CompanyRepository::new(pool.clone());
        let opp_repo = OpportunityRepository::new(pool.clone());
        let candidate_repo = CandidateRepository::new(pool.clone());
        let app_repo = ApplicationRepository::new(pool.clone());
        let saved_repo = SavedRepository::new(pool.clone());
        let discovery_repo = DiscoveryRepository::new(pool.clone());
        let gov_repo = GovernanceRepository::new(pool.clone());
        let taxonomy_repo = TaxonomyRepository::new(pool.clone());

        let token_service = TokenService::new(jwt_secret);

        let auth_use_cases = biz_application::identity::AuthUseCases::new(
            user_repo.clone(),
            token_repo.clone(),
            token_service.clone(),
        );
        let opp_use_cases = OpportunityUseCases::new(opp_repo.clone(), company_repo.clone());
        let discovery_use_cases = DiscoveryUseCases::new(discovery_repo, candidate_repo.clone());
        
        // اتصال یوزکیس کارجو به مخازن کارفرما برای قابلیت شکار استعداد و ارسال دعوت‌نامه
        let candidate_use_cases = CandidateUseCases::new(candidate_repo.clone(), app_repo.clone())
            .with_employer_repos(company_repo.clone(), opp_repo.clone());
            
        let app_use_cases = ApplicationUseCases::new(
            app_repo,
            opp_repo.clone(),
            candidate_repo.clone(),
            company_repo.clone(),
        );
        let saved_use_cases = SavedUseCases::new(saved_repo, candidate_repo.clone());
        let gov_use_cases = GovernanceUseCases::new(gov_repo.clone(), company_repo.clone());
        let company_use_cases = CompanyUseCases::new(company_repo, opp_repo, gov_repo);
        let taxonomy_use_cases = TaxonomyUseCases::new(taxonomy_repo);

        Self {
            pool,
            token_service,
            auth_use_cases,
            opp_use_cases,
            discovery_use_cases,
            candidate_use_cases,
            app_use_cases,
            saved_use_cases,
            gov_use_cases,
            company_use_cases,
            taxonomy_use_cases,
        }
    }

    pub fn discovery_repo_pool(&self) -> OpportunityRepository {
        OpportunityRepository::new(self.pool.clone())
    }
}