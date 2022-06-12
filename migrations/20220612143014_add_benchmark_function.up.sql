-- Add up migration script here
CREATE FUNCTION benchmark (query text, loop_times integer DEFAULT 1000)
RETURNS numeric
IMMUTABLE
AS $fn$
	DECLARE
		durations integer[];

		temp_timing timestamptz;
		start_ts timestamptz;
		end_ts timestamptz;
		overhead numeric;
		average_benchmark numeric;
	BEGIN
		temp_timing := clock_timestamp();
		start_ts := clock_timestamp();
		end_ts := clock_timestamp();

		-- take minimum duration as conservative estimate.
		overhead := 1000 * extract (epoch FROM least(start_ts - temp_timing, end_ts - start_ts));

		FOR i IN 1..loop_times  LOOP
			start_ts := clock_timestamp();
			EXECUTE query;
			end_ts := clock_timestamp();

			durations := durations || (1000 * (extract (epoch FROM (end_ts - start_ts))) - overhead);
		END LOOP;

		average_benchmark := (
			WITH durations_table AS (SELECT unnest(durations) AS duration)
			SELECT avg(duration) FROM durations_table
		);
	
		RETURN average_benchmark;
	END;
$fn$
LANGUAGE plpgsql;