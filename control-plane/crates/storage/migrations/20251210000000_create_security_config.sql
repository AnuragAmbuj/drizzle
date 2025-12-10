CREATE TABLE IF NOT EXISTS security_config (
    id INT PRIMARY KEY DEFAULT 1,
    global_rate_limit INT NOT NULL DEFAULT 100,
    global_burst INT NOT NULL DEFAULT 50,
    CONSTRAINT single_row CHECK (id = 1)
);

-- Insert default row if not exists
INSERT INTO security_config (id, global_rate_limit, global_burst)
VALUES (1, 100, 50)
ON CONFLICT (id) DO NOTHING;
