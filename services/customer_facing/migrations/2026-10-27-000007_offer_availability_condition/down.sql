ALTER TABLE offers DROP CONSTRAINT offers_availability_shape,
    DROP CONSTRAINT offers_item_condition_shape,
    DROP COLUMN availability, DROP COLUMN item_condition;
