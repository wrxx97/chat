-- Add migration script here
-- if user is added to chat, notify with chat data
CREATE OR REPLACE FUNCTION add_to_chat()
RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'add_to_chat: %', NEW;
    PERFORM pg_notify('chat_updated', json_build_object(
      'op', TG_OP,
      'old', OLD,
      'new', NEW
    )::TEXT);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_to_chat_trigger
AFTER INSERT OR UPDATE OR DELETE ON chats
FOR EACH ROW
EXECUTE FUNCTION add_to_chat();

-- if message is added to chat, notify with message data
CREATE OR REPLACE FUNCTION add_to_message()
RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'add_to_message: %', NEW;
    IF TG_OP = 'INSERT' THEN
        PERFORM pg_notify('message_updated', row_to_json(NEW)::TEXT);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_to_message_trigger
AFTER INSERT OR UPDATE OR DELETE ON messages
FOR EACH ROW
EXECUTE FUNCTION add_to_message();
