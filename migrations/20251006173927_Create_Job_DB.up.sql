-- Add up migration script here
CREATE TABLE company (
    id bigint PRIMARY KEY,
    name varchar(255) UNIQUE,
    logo_url varchar(1000)
);

CREATE Table job (
    id bigserial PRIMARY KEY,
    title varchar(255),
    description text,
    job_url text UNIQUE,
    company_id bigint,
    CONSTRAINT company_info_fk FOREIGN KEY (company_id)
    REFERENCES company(id)

);

CREATE TABLE job_tag (
    id bigserial PRIMARY KEY,
    tag varchar(255) UNIQUE
);


CREATE TABLE tags_for_job (
    job_id bigint,
    job_tag_id bigint,
    CONSTRAINT job_fk Foreign KEY (job_id) 
    REFERENCES job(id),
    CONSTRAINT job_tag_fk Foreign KEY (job_tag_id)  
    REFERENCES job_tag(id),

    CONSTRAINT tags_for_job_pk PRIMARY KEY (job_id,job_tag_id)

);


CREATE TABLE job_location (
    id bigserial PRIMARY KEY,
    address varchar(255),
    x double precision,
    y double precision,
    UNIQUE (x, y)
);
CREATE TABLE location_for_job (
    job_id bigint,
    location_id bigint,
    CONSTRAINT job_fk Foreign KEY (job_id) REFERENCES job(id),
    CONSTRAINT location_fk Foreign KEY (location_id)  REFERENCES job_location(id),

    CONSTRAINT location_for_job_pk PRIMARY KEY (job_id,location_id)

);






