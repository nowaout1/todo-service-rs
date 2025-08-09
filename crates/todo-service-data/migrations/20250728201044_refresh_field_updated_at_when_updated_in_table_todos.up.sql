CREATE OR REPLACE FUNCTION refresh_field_updated_at_when_updated_in_table_todos()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER refresh_field_updated_at_when_updated_in_table_todos
BEFORE UPDATE ON todos
FOR EACH ROW
EXECUTE FUNCTION refresh_field_updated_at_when_updated_in_table_todos();
