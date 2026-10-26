-- A retry after a timeout or crash must resolve to the original sale instead of
-- charging or discounting stock twice. The client sends a key per sale attempt
-- and the request hash detects reuse of the same key with different contents.
-- Existing sales keep NULL keys because their original client attempts cannot
-- be reconstructed.
ALTER TABLE sales ADD COLUMN idempotency_key TEXT;
ALTER TABLE sales ADD COLUMN request_hash TEXT;

CREATE UNIQUE INDEX idx_sales_organization_idempotency
    ON sales (organization_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
