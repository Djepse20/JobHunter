-- Add up migration script here
CREATE TABLE job_tag (
    id BIGINT PRIMARY KEY,
    job_tag VARCHAR(255)
);