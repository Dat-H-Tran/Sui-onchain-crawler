CREATE IF NOT EXISTS modules (
    id SERIAL PRIMARY KEY,
    package_id TEXT,
    sender TEXT,
    tx_digest TEXT,
    timestamp BIGINT,
    network_version TEXT,
    module_name TEXT,
    module_bytecode TEXT,
    module_hashed TEXT,
);

CREATE OR REPLACE FUNCTION hash_and_encode()
    RETURNS trigger
    LANGUAGE plpgsql AS
$func$
BEGIN
   NEW.module_hashed := encode(sha256(NEW.module_bytecode::bytea)::bytea, 'base64');
   RETURN NEW;
END
$func$;

CREATE TRIGGER hash_and_encode_bytecode
    BEFORE INSERT OR UPDATE OF module_bytecode ON modules
    FOR EACH ROW
    EXECUTE PROCEDURE hash_and_encode();
