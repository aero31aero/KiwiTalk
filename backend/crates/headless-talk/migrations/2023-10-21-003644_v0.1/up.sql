CREATE TABLE IF NOT EXISTS chat (
    log_id BIGINT PRIMARY KEY NOT NULL,
    channel_id BIGINT NOT NULL,
    prev_log_id BIGINT,
    type INTEGER NOT NULL,
    message_id BIGINT NOT NULL,
    send_at BIGINT NOT NULL,
    author_id BIGINT NOT NULL,
    message TEXT,
    attachment TEXT,
    supplement TEXT,
    referer INTEGER,

    deleted_time BIGINT
);

CREATE TABLE IF NOT EXISTS channel_list (
    id BIGINT PRIMARY KEY NOT NULL,
    type VARCHAR(16) NOT NULL,
    
    display_users VARCHAR(255) NOT NULL,

    active_user_count INTEGER NOT NULL,
    unread_count INTEGER NOT NULL,

    last_seen_log_id BIGINT,
    last_update BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS channel_meta (
    channel_id BIGINT NOT NULL,
    type INTEGER NOT NULL,

    author_id BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    revision BIGINT NOT NULL,
    content TEXT NOT NULL,

    PRIMARY KEY(channel_id, type)
);

CREATE TABLE IF NOT EXISTS user_profile (
    id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,

    nickname VARCHAR NOT NULL,

    profile_url TEXT NOT NULL,
    full_profile_url TEXT NOT NULL,
    original_profile_url TEXT NOT NULL,

    watermark BIGINT,

    PRIMARY KEY(id, channel_id)
);

CREATE TABLE IF NOT EXISTS normal_channel (
    id BIGINT PRIMARY KEY NOT NULL,

    joined_at_for_new_mem BIGINT,
    inviter_user_id BIGINT
);

CREATE TABLE IF NOT EXISTS normal_channel_user (
    id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,

    country_iso VARCHAR(4) NOT NULL,
    account_id BIGINT NOT NULL,
    status_message TEXT NOT NULL,
    linked_services TEXT NOT NULL,
    suspended BOOLEAN NOT NULL,
    
    PRIMARY KEY(id, channel_id)
);
