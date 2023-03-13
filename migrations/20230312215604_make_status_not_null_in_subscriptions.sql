BEGIN;
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status is NULL;
    -- Make status NOT NULL
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;

