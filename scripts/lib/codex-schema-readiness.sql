BEGIN READ ONLY;
WITH required(name) AS (VALUES
  ('room_events'), ('room_requests'), ('agent_conversations'),
  ('conversation_tasks'), ('conversation_updates'), ('conversation_disclosures'),
  ('conversation_update_requests')),
relations AS (
  SELECT c.oid, c.relname FROM pg_catalog.pg_class c
  JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
  JOIN required r ON r.name = c.relname
  WHERE n.nspname = 'public' AND c.relkind = 'r'
)
SELECT COALESCE(jsonb_object_agg(relname, jsonb_build_object(
  'columns', (SELECT jsonb_object_agg(a.attname, jsonb_build_object(
    'type', pg_catalog.format_type(a.atttypid, a.atttypmod),
    'nullable', NOT a.attnotnull,
    'default', pg_catalog.pg_get_expr(d.adbin, d.adrelid)))
    FROM pg_catalog.pg_attribute a LEFT JOIN pg_catalog.pg_attrdef d
      ON d.adrelid = a.attrelid AND d.adnum = a.attnum
    WHERE a.attrelid = relations.oid AND a.attnum > 0 AND NOT a.attisdropped),
  'constraints', (SELECT COALESCE(jsonb_agg(jsonb_build_object(
    'definition', pg_catalog.pg_get_constraintdef(c.oid, false),
    'validated', c.convalidated) ORDER BY pg_catalog.pg_get_constraintdef(c.oid, false)), '[]'::jsonb)
    FROM pg_catalog.pg_constraint c WHERE c.conrelid = relations.oid),
  'indexes', (SELECT COALESCE(jsonb_agg(jsonb_build_object(
    'definition', pg_catalog.pg_get_indexdef(i.indexrelid),
    'valid', i.indisvalid, 'ready', i.indisready) ORDER BY pg_catalog.pg_get_indexdef(i.indexrelid)), '[]'::jsonb)
    FROM pg_catalog.pg_index i WHERE i.indrelid = relations.oid)
)), '{}'::jsonb) FROM relations;
COMMIT;
