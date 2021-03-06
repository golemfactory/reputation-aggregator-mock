/*
Installation:
    PGPASSWORD=repu123 psql -U reputation -f t1.sql -1
    
    Subsequent execution overwrites results of the previous execution.

Refresh aggregates:
    SELECT calc.refresh_all()
    
    This takes ~ 1s for ~70000 rows in agreement_status.
    There are no clever optimalizations here - everything is recalculated.
    
Get score
    SELECT calc.standard_score('P', '0xabc123...')  # provider
    SELECT calc.standard_score('R', '0xabc123...')  # requestor

*/

DROP SCHEMA IF EXISTS calc CASCADE;
CREATE SCHEMA calc;
SET search_path TO calc;

-- Per-agreement aggregate
CREATE MATERIALIZED VIEW agreement AS
WITH
prefiltered_agreement_status AS (
        SELECT  s.role_id,
                s.node_id,
                s.agreement_id,
                s.requested,
                s.accepted,
                s.confirmed,
                s.created_ts,
                s.updated_ts,
                s.reported_ts,
                d.peer_id
        FROM    public.agreement_status  s
        JOIN    public.agreement_details d
            ON  s.agreement_id = d.agreement_id
),
provider AS (
    	SELECT 	*
	FROM 	prefiltered_agreement_status
	WHERE	role_id = 'P'
),
requestor AS (
    	SELECT 	*
	FROM 	prefiltered_agreement_status
	WHERE	role_id = 'R'
),
base_agg AS (
	-- This is an agreement "ground truth", i.e. information about the agreement without the
	-- "who exactly reported what" element.
	-- (Future TODO: we'll probably need a better way of solving conflicts than a simple average)
	SELECT 	coalesce(p.agreement_id, r.agreement_id) AS agreement_id,
		coalesce(p.node_id, r.peer_id) AS p_id,
		coalesce(r.node_id, p.peer_id) AS r_id,
		coalesce((p.requested + r.requested)/2, p.requested, r.requested) AS requested,
		coalesce((p.accepted  + r.accepted) /2, p.accepted,  r.accepted)  AS accepted,
		coalesce((p.confirmed + r.confirmed)/2, p.confirmed, r.confirmed) AS confirmed,
		greatest(p.updated_ts, r.updated_ts) AS updated_ts
	FROM	provider  p
	FULL
	OUTER
	JOIN	requestor r
	  ON p.agreement_id = r.agreement_id
),
agreement_type AS (
	SELECT	ba.*,
		CASE 	WHEN requested = 0
			THEN 'CANCELLED'

			-- NOTE about the '>=' - we expect '=', but if requestor accepts/pays more then 
			-- it's also probably fine?
			WHEN accepted >= requested AND confirmed >= accepted
		  	THEN 'PAID'
			WHEN accepted > confirmed
		  	THEN 'BAD_REQUESTOR'
			WHEN requested > accepted AND accepted > 0
		  	THEN 'AGREEMENT_BROKEN'
			WHEN requested = 0
			THEN 'AGREEMENT_CANCELLED'
			WHEN accepted = 0
		  	THEN 'AGREEMENT_FAILED'
			ELSE '???'
		END AS agreement_result
	FROM	base_agg ba
	-- TODO: this should not be possible, this should change when we have "peer_id NOT NULL"
	WHERE 	ba.p_id IS NOT NULL 
	   AND  ba.r_id IS NOT NULL
),
array_scores AS (
	--	NOTE: the only purpose of this aggregate is to have provider and requestor scores
	--	      next to each other, array is unpacked later
	SELECT	at.*,
		CASE
			WHEN agreement_result = 'PAID'
			THEN ARRAY[accepted, accepted]
			WHEN agreement_result = 'BAD_REQUESTOR'
			THEN ARRAY[0, (confirmed - accepted) * 10]
			WHEN agreement_result = 'AGREEMENT_BROKEN'
			THEN ARRAY[accepted * 0.9, accepted * 0.9]
			WHEN agreement_result = 'AGREEMENT_FAILED'
			-- TODO: this is just some arbitrary number
			THEN ARRAY[-0.01, -0.01]
			ELSE ARRAY[0, 0]
		END AS scores
	FROM	agreement_type at
)
SELECT	a.agreement_id, a.p_id, a.r_id, a.requested, a.accepted, a.confirmed, a.agreement_result, a.updated_ts,
	scores[1] AS p_score,
	scores[2] AS r_score
FROM	array_scores a
;

CREATE MATERIALIZED VIEW node_score AS
WITH
raw_score AS (
	SELECT	a.p_id		AS node_id,
		'P' 		AS role_id,
		sum(p_score)	AS raw_score
	FROM	calc.agreement a
	GROUP BY 1
	UNION
	SELECT	a.r_id		AS node_id,
		'R'		AS role_id,
		sum(r_score)	AS raw_score
	FROM	calc.agreement a
	GROUP BY 1
),
agg_metrics AS (
	SELECT	rs.role_id,
 		avg(rs.raw_score) AS avg,
 	        stddev_samp(rs.raw_score) AS stddev
	FROM	raw_score rs
	GROUP BY 1
)
SELECT	rs.node_id,
	rs.role_id,
	round(rs.raw_score, 8) AS raw_score,
	round((rs.raw_score - am.avg) / am.stddev, 8) AS standard_score
FROM	raw_score 	rs
JOIN	agg_metrics 	am
    ON	rs.role_id = am.role_id
;

CREATE UNIQUE INDEX ON node_score (role_id, node_id);

CREATE FUNCTION refresh_all() RETURNS void
LANGUAGE plpgsql
AS $fff$
BEGIN
	REFRESH MATERIALIZED VIEW calc.agreement;
	REFRESH MATERIALIZED VIEW calc.node_score;
END;
$fff$
;

CREATE FUNCTION standard_score(role_id CHAR(1), node_id VARCHAR(42)) RETURNS numeric
LANGUAGE SQL
AS $fff$
	SELECT 	standard_score
	FROM	calc.node_score ns
	WHERE	(ns.role_id, ns.node_id) = ($1, $2);
$fff$
;
