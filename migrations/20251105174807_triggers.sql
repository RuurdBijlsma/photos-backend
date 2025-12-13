-- Function to broadcast a notification when a row is inserted
CREATE OR REPLACE FUNCTION notify_new_media_item() RETURNS trigger AS
$$
BEGIN
    PERFORM pg_notify('media_item_added', row_to_json(NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_new_media_item
    AFTER INSERT
    ON media_item
    FOR EACH ROW
EXECUTE FUNCTION notify_new_media_item();


-- =========================================================================================
-- Album Timestamp Triggers (Statement Level)
-- =========================================================================================

CREATE OR REPLACE FUNCTION update_album_latest_timestamp_stmt()
    RETURNS TRIGGER AS
$$
BEGIN
    -- Handle INSERTS
    IF (TG_OP = 'INSERT') THEN
        UPDATE album a
        SET latest_media_item_timestamp = (
            -- This subquery runs EXACTLY ONCE per distinct album_id
            SELECT MAX(mi.sort_timestamp)
            FROM album_media_item ami
                     JOIN media_item mi ON ami.media_item_id = mi.id
            WHERE ami.album_id = a.id
              AND mi.deleted = false)
        -- !!! CRITICAL FIX: Use DISTINCT to ensure we only update each album ONCE per batch !!!
        FROM (SELECT DISTINCT album_id FROM new_table) nt
        WHERE a.id = nt.album_id;

        -- Handle DELETES
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE album a
        SET latest_media_item_timestamp = (SELECT MAX(mi.sort_timestamp)
                                           FROM album_media_item ami
                                                    JOIN media_item mi ON ami.media_item_id = mi.id
                                           WHERE ami.album_id = a.id
                                             AND mi.deleted = false)
        -- !!! CRITICAL FIX: Use DISTINCT here too !!!
        FROM (SELECT DISTINCT album_id FROM old_table) ot
        WHERE a.id = ot.album_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger for INSERTS
CREATE TRIGGER trigger_update_album_timestamp_insert
    AFTER INSERT
    ON album_media_item
    REFERENCING NEW TABLE AS new_table
    FOR EACH STATEMENT
EXECUTE FUNCTION update_album_latest_timestamp_stmt();

-- Trigger for DELETES
CREATE TRIGGER trigger_update_album_timestamp_delete
    AFTER DELETE
    ON album_media_item
    REFERENCING OLD TABLE AS old_table
    FOR EACH STATEMENT
EXECUTE FUNCTION update_album_latest_timestamp_stmt();