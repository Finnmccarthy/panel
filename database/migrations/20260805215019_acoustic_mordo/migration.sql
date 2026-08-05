-- Custom SQL migration file, put your code below! --

-- The `format` schedule step switched from single-brace to double-brace
-- placeholders, so that json (and anything else brace-heavy) can be written
-- literally without escaping:
--
--   old: {{"content": "{message}"}}
--   new: {"content": "{{message}}"}
--
-- Under the old syntax `{name}` interpolated, `{{` was a literal `{` and `}}` a
-- literal `}`. Under the new one `{{name}}` interpolates and single braces are
-- literal. This replays the old scanner over every stored format string and
-- re-emits it in the new syntax, so existing steps keep producing what they
-- produced before.
CREATE FUNCTION pg_temp.rewrite_schedule_format(format text) RETURNS text AS $$
DECLARE
	rewritten text := '';
	idx int := 1;
	total int;
	this_char text;
	next_char text;
	closing int;
	variable text;
BEGIN
	IF format IS NULL THEN
		RETURN format;
	END IF;

	total := length(format);

	WHILE idx <= total LOOP
		this_char := substr(format, idx, 1);
		next_char := CASE WHEN idx < total THEN substr(format, idx + 1, 1) END;

		IF this_char = '{' AND next_char = '{' THEN
			-- escaped literal brace
			rewritten := rewritten || '{';
			idx := idx + 2;
		ELSIF this_char = '{' THEN
			closing := strpos(substr(format, idx + 1), '}');

			IF closing > 0 THEN
				variable := substr(format, idx + 1, closing - 1);

				-- Only convert things that could plausibly have been a variable
				-- name. Anything else (most commonly json that was typed in
				-- raw) never interpolated under the old scanner either, and
				-- must keep rendering verbatim -- which single braces now do.
				IF variable ~ '^[[:space:]]*[A-Za-z0-9_][A-Za-z0-9_. -]*[[:space:]]*$' THEN
					rewritten := rewritten || '{{' || variable || '}}';
				ELSE
					rewritten := rewritten || '{' || variable || '}';
				END IF;

				idx := idx + closing + 1;
			ELSE
				-- unterminated placeholder, emitted verbatim by the old scanner
				rewritten := rewritten || substr(format, idx);
				idx := total + 1;
			END IF;
		ELSIF this_char = '}' AND next_char = '}' THEN
			rewritten := rewritten || '}';
			idx := idx + 2;
		ELSE
			rewritten := rewritten || this_char;
			idx := idx + 1;
		END IF;
	END LOOP;

	RETURN rewritten;
END;
$$ LANGUAGE plpgsql;

UPDATE server_schedule_steps
SET action = jsonb_set(
	action,
	'{format}',
	to_jsonb(pg_temp.rewrite_schedule_format(action->>'format'))
)
WHERE action->>'type' = 'format'
	AND jsonb_typeof(action->'format') = 'string';
