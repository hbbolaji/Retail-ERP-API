-- Add migration script here
CREATE TABLE IF NOT EXISTS products (
  id SERIAL PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  barcode VARCHAR(100) NOT NULL,
  source VARCHAR(100) NOT NULL,
  category VARCHAR(100) NOT NULL,
  manufacturer VARCHAR(100) NOT NULL,
  nafdac_reg_no VARCHAR(100),
  nafdac_expiration_date TIMESTAMPTZ,
  nafdac_approval_date TIMESTAMPTZ, 
  created_at TIMESTAMPTZ DEFAULT NOW()
)