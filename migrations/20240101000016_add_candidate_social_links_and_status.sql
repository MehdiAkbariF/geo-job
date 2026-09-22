-- Migration: Add professional social links and job search status to candidates table
ALTER TABLE candidates 
ADD COLUMN IF NOT EXISTS linkedin_url VARCHAR(255),
ADD COLUMN IF NOT EXISTS github_url VARCHAR(255),
ADD COLUMN IF NOT EXISTS website_url VARCHAR(255),
ADD COLUMN IF NOT EXISTS job_search_status VARCHAR(32) NOT NULL DEFAULT 'actively_looking';