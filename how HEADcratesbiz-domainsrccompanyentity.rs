[1mdiff --git a/crates/biz-domain/src/company/entity.rs b/crates/biz-domain/src/company/entity.rs[m
[1mindex ff23622..2737cf3 100644[m
[1m--- a/crates/biz-domain/src/company/entity.rs[m
[1m+++ b/crates/biz-domain/src/company/entity.rs[m
[36m@@ -1,11 +1,12 @@[m
[31m-use super::membership::CompanyRole;[m
[32m+[m[32muse crate::error::DomainError;[m
 use chrono::{DateTime, Utc};[m
 use serde::{Deserialize, Serialize};[m
[32m+[m[32muse utoipa::ToSchema;[m
 use uuid::Uuid;[m
 [m
[31m-#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)][m
[32m+[m[32m#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)][m
 #[serde(rename_all = "snake_case")][m
[31m-pub enum CompanyVerificationStatus {[m
[32m+[m[32mpub enum VerificationStatus {[m
     NotStarted,[m
     Pending,[m
     UnderReview,[m
[36m@@ -14,13 +15,7 @@[m [mpub enum CompanyVerificationStatus {[m
     Expired,[m
 }[m
 [m
[31m-impl Default for CompanyVerificationStatus {[m
[31m-    fn default() -> Self {[m
[31m-        Self::NotStarted[m
[31m-    }[m
[31m-}[m
[31m-[m
[31m-#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)][m
[32m+[m[32m#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)][m
 pub struct Company {[m
     pub id: Uuid,[m
     pub name: String,[m
[36m@@ -28,17 +23,9 @@[m [mpub struct Company {[m
     pub description: Option<String>,[m
     pub logo_storage_key: Option<String>,[m
     pub website: Option<String>,[m
[31m-    pub verification_status: CompanyVerificationStatus,[m
[31m-    pub created_at: DateTime<Utc>,[m
[31m-    pub updated_at: DateTime<Utc>,[m
[31m-}[m
[31m-[m
[31m-#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)][m
[31m-pub struct CompanyMembership {[m
[31m-    pub id: Uuid,[m
[31m-    pub company_id: Uuid,[m
[31m-    pub user_id: Uuid,[m
[31m-    pub role: CompanyRole,[m
[32m+[m[32m    pub business_type: String, // corporate, retail_shop, restaurant_cafe, clinic_office, workshop[m
[32m+[m[32m    pub trade_license_number: Option<String>,[m
[32m+[m[32m    pub verification_status: VerificationStatus,[m
     pub created_at: DateTime<Utc>,[m
     pub updated_at: DateTime<Utc>,[m
 }[m
[36m@@ -49,4 +36,18 @@[m [mpub struct NewCompany {[m
     pub slug: String,[m
     pub description: Option<String>,[m
     pub website: Option<String>,[m
[32m+[m[32m    pub business_type: String,[m
[32m+[m[32m    pub trade_license_number: Option<String>,[m
[32m+[m[32m}[m
[32m+[m
[32m+[m[32mimpl NewCompany {[m
[32m+[m[32m    pub fn validate(&self) -> Result<(), DomainError> {[m
[32m+[m[32m        if self.name.trim().is_empty() {[m
[32m+[m[32m            return Err(DomainError::Validation("نام کسب‌وکار یا سازمان نمی‌تواند خالی باشد".into()));[m
[32m+[m[32m        }[m
[32m+[m[32m        if self.slug.trim().is_empty() {[m
[32m+[m[32m            return Err(DomainError::Validation("شناسه انگلیسی (Slug) الزامی است".into()));[m
[32m+[m[32m        }[m
[32m+[m[32m        Ok(())[m
[32m+[m[32m    }[m
 }[m
\ No newline at end of file[m
