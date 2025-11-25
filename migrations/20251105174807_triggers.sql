-- Function to broadcast a notification when a row is inserted
CREATE OR REPLACE FUNCTION notify_new_media_item() RETURNS trigger AS $$
BEGIN
    -- We send the new row as a JSON string payload. Maybe later send only id, but IDK what's necessary yet.
    PERFORM pg_notify('media_item_added', row_to_json(NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger calling the function on INSERT
CREATE TRIGGER trigger_new_media_item
    AFTER INSERT ON media_item
    FOR EACH ROW
EXECUTE FUNCTION notify_new_media_item();