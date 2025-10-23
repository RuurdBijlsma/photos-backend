CREATE TABLE monthly_photo_ratios (
                                      user_id INT NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
                                      month_start TIMESTAMPTZ NOT NULL,
                                      ratios REAL[] NOT NULL,
                                      PRIMARY KEY (user_id, month_start)
);
CREATE INDEX idx_monthly_photo_ratios_user_id_month_start_desc ON monthly_photo_ratios (user_id, month_start DESC);



CREATE OR REPLACE FUNCTION update_monthly_photo_ratios()
    RETURNS TRIGGER AS $$
DECLARE
    v_user_id INT;
    v_month_start TIMESTAMPTZ;
BEGIN
    -- Determine the user_id and month_start from the old or new row.
    IF TG_OP = 'DELETE' THEN
        v_user_id := OLD.user_id;
        v_month_start := DATE_TRUNC('month', OLD.taken_at_local);
    ELSE
        v_user_id := NEW.user_id;
        v_month_start := DATE_TRUNC('month', NEW.taken_at_local);
    END IF;

    -- If an update happens and the month has changed, we need to update the old month's data as well.
    IF TG_OP = 'UPDATE' AND DATE_TRUNC('month', OLD.taken_at_local) <> v_month_start THEN
        PERFORM update_single_month_ratios(OLD.user_id, DATE_TRUNC('month', OLD.taken_at_local));
    END IF;

    -- Update the data for the affected month.
    PERFORM update_single_month_ratios(v_user_id, v_month_start);

    RETURN NULL; -- The result is ignored since this is an AFTER trigger.
END;
$$ LANGUAGE plpgsql;



-- A helper function to recalculate the ratios for a specific user and month.
CREATE OR REPLACE FUNCTION update_single_month_ratios(p_user_id INT, p_month_start TIMESTAMPTZ)
    RETURNS void AS $$
BEGIN
    -- Use an UPSERT (INSERT ... ON CONFLICT ...) to either create a new row or update an existing one.
    INSERT INTO monthly_photo_ratios (user_id, month_start, ratios)
    SELECT
        p_user_id,
        p_month_start,
        COALESCE(
                array_agg((width::float / height)::real ORDER BY taken_at_local DESC),
                '{}'::real[]
        )
    FROM
        media_item
    WHERE
        user_id = p_user_id
      AND DATE_TRUNC('month', taken_at_local) = p_month_start
      AND deleted = false
    ON CONFLICT (user_id, month_start) DO UPDATE
        SET ratios = EXCLUDED.ratios;

    -- If no photos are left for a given month, the aggregation will produce an empty array.
    -- We can remove the row from the summary table to keep it clean.
    DELETE FROM monthly_photo_ratios
    WHERE user_id = p_user_id AND month_start = p_month_start AND ratios = '{}'::real[];

END;
$$ LANGUAGE plpgsql;



-- Trigger for after a new media_item is inserted.
CREATE TRIGGER media_item_after_insert
    AFTER INSERT ON media_item
    FOR EACH ROW
EXECUTE FUNCTION update_monthly_photo_ratios();

-- Trigger for after a media_item is updated.
-- We only need to run the trigger if relevant columns change.
CREATE TRIGGER media_item_after_update
    AFTER UPDATE ON media_item
    FOR EACH ROW
    WHEN (OLD.deleted IS DISTINCT FROM NEW.deleted OR OLD.taken_at_local IS DISTINCT FROM NEW.taken_at_local)
EXECUTE FUNCTION update_monthly_photo_ratios();

-- Trigger for after a media_item is deleted.
CREATE TRIGGER media_item_after_delete
    AFTER DELETE ON media_item
    FOR EACH ROW
EXECUTE FUNCTION update_monthly_photo_ratios();