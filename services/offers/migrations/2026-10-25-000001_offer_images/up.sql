CREATE TABLE read_offer_images (
    offer_id UUID NOT NULL REFERENCES read_offers(id),
    image_url TEXT COLLATE "C" NOT NULL CHECK (octet_length(image_url) <= 2048),
    position BIGINT NOT NULL CHECK (position BETWEEN 0 AND 4294967295),
    PRIMARY KEY (offer_id, position),
    UNIQUE (offer_id, image_url)
);
