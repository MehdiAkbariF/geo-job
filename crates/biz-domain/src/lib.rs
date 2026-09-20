pub mod candidate;
pub mod company;
pub mod error;
pub mod identity;
pub mod opportunity;
pub mod taxonomy;

pub use candidate::{Candidate, CandidateEducation, CandidateExperience, CandidateResume};
pub use company::{Company, CompanyMembership, CompanyRole, CompanyVerificationStatus, NewCompany};
pub use error::DomainError;
pub use identity::{Email, NewUser, RawPassword, User, UserStatus};
pub use opportunity::{NewOpportunity, Opportunity, OpportunityStatus, OpportunityType, RemoteScope, Salary, WorkplaceType};
pub use taxonomy::{Category, ExperienceLevel, Industry, Occupation, Skill, SkillAlias};