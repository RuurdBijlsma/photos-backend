-- 1. Create the function that performs the update for a specific ID
CREATE
    OR REPLACE FUNCTION rebuild_media_item_search_vector(target_id VARCHAR(10))
    RETURNS VOID AS
$$
BEGIN
    UPDATE media_item
    SET search_vector = (SELECT
                             -- HIGH CONFIDENCE ('A')
                             setweight(to_tsvector('english', mi.filename), 'A') ||
                             setweight(to_tsvector('english', coalesce(l.name, '')), 'A') ||
                             setweight(to_tsvector('english', coalesce(l.admin1, '')), 'A') ||
                             setweight(to_tsvector('english', coalesce(l.country_name, '')), 'A') ||
                             setweight(to_tsvector('english', coalesce(p_agg.names, '')), 'A') ||
                                 -- MEDIUM CONFIDENCE ('B')
                             setweight(to_tsvector('english', coalesce(l.admin2, '')), 'B') ||
                             setweight(to_tsvector('english', coalesce(c.ocr_text, '')), 'B') ||
                             setweight(to_tsvector('english', coalesce(w.condition, '')), 'B') ||
                             setweight(to_tsvector('english', coalesce(c.event_type, '')), 'B') ||
                             setweight(to_tsvector('english', coalesce(c.setting, '')), 'B') ||
                             setweight(to_tsvector('english', coalesce(c.landmark_name, '')), 'B') ||
                             setweight(to_tsvector('english', coalesce(c.caption, '')), 'B') ||
                                 -- LOWER CONFIDENCE ('C' & 'D')
                             setweight(to_tsvector('english', to_char(mi.taken_at_local, 'YYYY Month Day')), 'C') ||
                             setweight(to_tsvector('english', CASE WHEN mi.is_video THEN 'video' ELSE 'photo' END), 'D')
                         FROM media_item mi
                                  LEFT JOIN gps g ON mi.id = g.media_item_id
                                  LEFT JOIN location l ON g.location_id = l.id
                                  LEFT JOIN weather w ON mi.id = w.media_item_id
                                  LEFT JOIN visual_analysis va ON mi.id = va.media_item_id
                                  LEFT JOIN classification c ON va.id = c.visual_analysis_id
                                  LEFT JOIN (SELECT va_inner.media_item_id, string_agg(pers.name, ' ') as names
                                             FROM face f
                                                      JOIN person pers
                                                           ON f.person_id = pers.id
                                                      JOIN visual_analysis va_inner ON f.visual_analysis_id = va_inner.id
                                             GROUP BY va_inner.media_item_id) p_agg ON mi.id = p_agg.media_item_id
                         WHERE mi.id = target_id)
    WHERE id = target_id;
END;
$$
    LANGUAGE plpgsql;

-- 2. Trigger function to call the rebuilder
CREATE
    OR REPLACE FUNCTION tg_rebuild_search_vector()
    RETURNS TRIGGER AS
$$
BEGIN
    IF
        TG_TABLE_NAME = 'media_item' THEN
        PERFORM rebuild_media_item_search_vector(NEW.id);
    ELSIF
        TG_TABLE_NAME = 'gps' OR TG_TABLE_NAME = 'weather' OR TG_TABLE_NAME = 'visual_analysis' THEN
        PERFORM rebuild_media_item_search_vector(NEW.media_item_id);
    ELSIF
        TG_TABLE_NAME = 'classification' THEN
        -- Link back via visual_analysis
        PERFORM rebuild_media_item_search_vector((SELECT media_item_id
                                                  FROM visual_analysis
                                                  WHERE id = NEW.visual_analysis_id));
    ELSIF
        TG_TABLE_NAME = 'face' THEN
        PERFORM rebuild_media_item_search_vector((SELECT media_item_id
                                                  FROM visual_analysis
                                                  WHERE id = NEW.visual_analysis_id));
    END IF;
    RETURN NEW;
END;
$$
    LANGUAGE plpgsql;

-- 3. Apply triggers to tables
CREATE TRIGGER trg_mi_search_update
    AFTER INSERT OR UPDATE OF relative_path, taken_at_local
    ON media_item
    FOR EACH ROW
EXECUTE FUNCTION tg_rebuild_search_vector();
CREATE TRIGGER trg_gps_search_update
    AFTER INSERT OR UPDATE
    ON gps
    FOR EACH ROW
EXECUTE FUNCTION tg_rebuild_search_vector();
CREATE TRIGGER trg_weather_search_update
    AFTER INSERT OR UPDATE
    ON weather
    FOR EACH ROW
EXECUTE FUNCTION tg_rebuild_search_vector();
CREATE TRIGGER trg_va_search_update
    AFTER INSERT OR UPDATE
    ON visual_analysis
    FOR EACH ROW
EXECUTE FUNCTION tg_rebuild_search_vector();
CREATE TRIGGER trg_classification_search_update
    AFTER INSERT OR UPDATE
    ON classification
    FOR EACH ROW
EXECUTE FUNCTION tg_rebuild_search_vector();
CREATE TRIGGER trg_face_search_update
    AFTER INSERT OR UPDATE
    ON face
    FOR EACH ROW
EXECUTE FUNCTION tg_rebuild_search_vector();

-- Special case: If a Person's name changes, we need to update ALL media_items containing that person
CREATE
    OR REPLACE FUNCTION tg_person_rename_search_update()
    RETURNS TRIGGER AS
$$
BEGIN
    IF
        OLD.name IS DISTINCT FROM NEW.name THEN
        PERFORM rebuild_media_item_search_vector(va.media_item_id)
        FROM face f
                 JOIN visual_analysis va ON f.visual_analysis_id = va.id
        WHERE f.person_id = NEW.id;
    END IF;
    RETURN NEW;
END;
$$
    LANGUAGE plpgsql;

CREATE TRIGGER trg_person_search_update
    AFTER UPDATE OF name
    ON person
    FOR EACH ROW
EXECUTE FUNCTION tg_person_rename_search_update();

-- todo check if i got proper indices for these sql queries