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
        -- ! Use DISTINCT to ensure we only update each album ONCE per batch !
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
        -- ! Use DISTINCT here too !
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


-- Trigger for album.media_count

CREATE OR REPLACE FUNCTION update_album_media_count_stmt()
    RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE album a
        SET media_count = media_count + sub.cnt
        FROM (
                 -- Count only items being inserted that aren't soft-deleted
                 SELECT nt.album_id, COUNT(*) as cnt
                 FROM new_table nt
                          JOIN media_item mi ON nt.media_item_id = mi.id
                 WHERE mi.deleted = false
                 GROUP BY nt.album_id
             ) sub
        WHERE a.id = sub.album_id;

    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE album a
        SET media_count = media_count - sub.cnt
        FROM (
                 -- Count only items being removed that weren't soft-deleted
                 SELECT ot.album_id, COUNT(*) as cnt
                 FROM old_table ot
                          JOIN media_item mi ON ot.media_item_id = mi.id
                 WHERE mi.deleted = false
                 GROUP BY ot.album_id
             ) sub
        WHERE a.id = sub.album_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Replace the previous trg_album_media_item_count with this:
DROP TRIGGER IF EXISTS trg_album_media_item_count ON album_media_item;

CREATE TRIGGER trg_album_media_item_count_insert
    AFTER INSERT ON album_media_item
    REFERENCING NEW TABLE AS new_table
    FOR EACH STATEMENT EXECUTE FUNCTION update_album_media_count_stmt();

CREATE TRIGGER trg_album_media_item_count_delete
    AFTER DELETE ON album_media_item
    REFERENCING OLD TABLE AS old_table
    FOR EACH STATEMENT EXECUTE FUNCTION update_album_media_count_stmt();

CREATE OR REPLACE FUNCTION fn_trigger_media_item_hard_delete_sync()
    RETURNS TRIGGER AS $$
BEGIN
    IF (OLD.deleted = false) THEN
        UPDATE album
        SET media_count = media_count - 1
        WHERE id IN (
            SELECT album_id
            FROM album_media_item
            WHERE media_item_id = OLD.id
        );
    END IF;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_media_item_hard_delete
    BEFORE DELETE ON media_item
    FOR EACH ROW
EXECUTE FUNCTION fn_trigger_media_item_hard_delete_sync();