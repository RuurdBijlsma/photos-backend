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


-- Album Trigger

-- Function to recalculate the latest_media_item_timestamp on the parent album
CREATE OR REPLACE FUNCTION update_album_latest_timestamp()
    RETURNS TRIGGER AS $$
DECLARE
    target_album_id VARCHAR(10);
BEGIN
    IF (TG_OP = 'DELETE') THEN
        target_album_id := OLD.album_id;
    ELSE
        target_album_id := NEW.album_id;
    END IF;

    UPDATE album
    SET latest_media_item_timestamp = (
        SELECT MAX(mi.sort_timestamp)
        FROM album_media_item ami
                 JOIN media_item mi ON ami.media_item_id = mi.id
        WHERE ami.album_id = target_album_id
          AND mi.deleted = false
    )
    WHERE id = target_album_id;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger to fire the function whenever the album content changes
CREATE TRIGGER trigger_update_album_timestamp
    AFTER INSERT OR DELETE OR UPDATE ON album_media_item
    FOR EACH ROW
EXECUTE FUNCTION update_album_latest_timestamp();