CREATE TABLE IF NOT EXISTS sui_packages (
    id SERIAL PRIMARY KEY,
    package_id TEXT UNIQUE,
    sender TEXT,
    tx_digest TEXT,
    timestamp BIGINT,
    network_version TEXT,
    content TEXT
);

-- CREATE TABLE IF NOT EXISTS sui_modules (
--     id SERIAL PRIMARY KEY,
--     package_id TEXT REFERENCES sui_packages(package_id),
--     name TEXT,
--     bytecode TEXT,
--     bytecode_digest TEXT
-- );

-- CREATE OR REPLACE FUNCTION hash_and_encode()
--     RETURNS trigger
--     LANGUAGE plpgsql AS
-- $func$
-- BEGIN
--    NEW.module_hashed := encode(sha256(NEW.module_bytecode::bytea)::bytea, 'base64');
--    RETURN NEW;
-- END
-- $func$;

-- CREATE TRIGGER hash_and_encode_bytecode
--     BEFORE INSERT OR UPDATE OF module_bytecode ON modules
--     FOR EACH ROW
--     EXECUTE PROCEDURE hash_and_encode();
