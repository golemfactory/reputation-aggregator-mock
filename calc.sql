DROP SCHEMA IF EXISTS calc CASCADE;
CREATE SCHEMA calc;
SET search_path TO calc;

-- Per-agreement aggregate
CREATE TABLE agreement AS
WITH
provider AS (
    	SELECT 	*
	FROM 	public.agreement_status
	WHERE	role_id = 'P'
),
requestor AS (
    	SELECT 	*
	FROM 	public.agreement_status
	WHERE	role_id = 'R'
),
base_agg AS (
	SELECT 	coalesce(p.agreement_id, r.agreement_id) AS agreement_id,
		coalesce(p.node_id, r.peer_id) AS p_id,
		coalesce(r.node_id, p.peer_id) AS r_id,
		coalesce((p.requested + r.requested)/2, p.requested, r.requested) AS requested,
		coalesce((p.accepted  + r.accepted) /2, p.accepted,  r.accepted)  AS accepted,
		coalesce((p.confirmed + r.confirmed)/2, p.confirmed, r.confirmed) AS confirmed
	FROM	provider  p
	FULL
	OUTER
	JOIN	requestor r
	  ON p.agreement_id = r.agreement_id
),
agreement_type AS (
	SELECT	ba.*,
		CASE 
			WHEN requested = accepted AND accepted = confirmed
		  	THEN 'PAID'
			WHEN accepted > confirmed
		  	THEN 'BAD_REQUESTOR'
			WHEN requested > accepted AND accepted > 0
		  	THEN 'AGREEMENT_BROKEN'
			WHEN accepted = 0
		  	THEN 'AGREEMENT_FAILED'
			ELSE '???'
		END AS agreement_result
	FROM	base_agg ba
	-- TODO: how is this possible?
	-- Why peer_id can be NULL?
	WHERE 	ba.p_id IS NOT NULL 
	   AND  ba.r_id IS NOT NULL
)
SELECT	at.*,
	CASE
		WHEN agreement_result = 'PAID'
		THEN ARRAY[accepted, accepted]
		WHEN agreement_result = 'BAD_REQUESTOR'
		THEN ARRAY[0, (confirmed - accepted) * 10]
		WHEN agreement_result = 'AGREEMENT_BROKEN'
		THEN ARRAY[accepted * 0.9, accepted * 0.9]
		WHEN agreement_result = 'AGREEMENT_FAILED'
		THEN ARRAY[-0.01, -0.01]
		ELSE ARRAY[0, 0]
	END AS scores
FROM	agreement_type at
;


-- Score aggregation
CREATE TABLE scores (
  id		SERIAL PRIMARY KEY,
  node_id	VARCHAR(42) NOT NULL,
  role_id 	CHARACTER(1) NOT NULL CHECK (role_id = ANY (ARRAY['R'::bpchar, 'P'::bpchar])),
  agreement_id  VARCHAR(120) NOT NULL,
  score		numeric NOT NULL
);


INSERT INTO scores (node_id, role_id, agreement_id, score)
SELECT	p_id, 'P', agreement_id, scores[1]
FROM	agreement
UNION
SELECT	r_id, 'R', agreement_id, scores[2]
FROM	agreement
;

CREATE TABLE node_score (
  node_id	VARCHAR(42) NOT NULL,
  role_id 	CHARACTER(1) NOT NULL CHECK (role_id = ANY (ARRAY['R'::bpchar, 'P'::bpchar])),
  raw_score	numeric,
  standarized_score numeric,
  PRIMARY KEY (node_id, role_id)
);
 
INSERT INTO node_score (node_id, role_id, raw_score)
SELECT	node_id,
	role_id,
	round(sum(score), 4)
FROM	scores
GROUP BY 1, 2
ORDER BY 2, 1
;

WITH
avg_stddev AS (
	SELECT	role_id,
		avg(raw_score) AS avg,
	        stddev_samp(raw_score) AS stddev
	FROM	node_score
	GROUP BY 1
)
UPDATE 	node_score
SET	standarized_score = round((raw_score - a.avg) / a.stddev, 4)
FROM	avg_stddev a
WHERE	node_score.role_id = a.role_id
;

-- SELECT * FROM node_score ORDER BY role_id, standarized_score DESC;

