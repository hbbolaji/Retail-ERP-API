-- Add migration script here
CREATE TABLE IF NOT EXISTS products (
  id SERIAL PRIMARY KEY,
  name VARCHAR(200) NOT NULL,
  barcode VARCHAR(200),
  source VARCHAR(200),
  category VARCHAR(200),
  manufacturer VARCHAR(200),
  nafdac_reg_no VARCHAR(200),
  nafdac_expiration_date TIMESTAMPTZ,
  nafdac_approval_date TIMESTAMPTZ, 
  created_at TIMESTAMPTZ DEFAULT NOW()
)
