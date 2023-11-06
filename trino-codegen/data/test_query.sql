SELECT
    -- Exact numeric types
    CAST(
        1 AS INTEGER
    ) AS integer_type,
    CAST(
        1 AS SMALLINT
    ) AS smallint_type,
    CAST(
        1 AS tinyint
    ) AS tinyint_type,
    CAST(12345678901234567890 AS DECIMAL(38, 0)) AS decimal_type,
    -- Approximate numeric types
    CAST(
        1.2345 AS REAL
    ) AS real_type,
    CAST(
        1.234567890123456789 AS DOUBLE
    ) AS double_type,
    -- Boolean type
    CAST(
        TRUE AS BOOLEAN
    ) AS boolean_type,
    -- Date and Time types
    CAST(
        DATE '2023-01-01' AS DATE
    ) AS date_type,
    CAST(
        '2023-01-01 01:02:03.456' AS TIMESTAMP
    ) AS timestamp_type,
    CAST(
        '2023-01-01 01:02:03.456 UTC' AS TIMESTAMP WITH TIME ZONE
    ) AS timestamptz_type,
    CAST(
        '01:02:03.456' AS TIME
    ) AS time_type,
    -- Character types
    CAST(
        'a' AS CHAR
    ) AS char_type,
    CAST(
        'a variable-length string' AS VARCHAR
    ) AS varchar_type,
    -- Binary data types (assuming hex format for binary literals)
    CAST(
        'DEADBEEF' AS varbinary
    ) AS varbinary_type,
    -- JSON type
    CAST(
        '{ "key": "value" }' AS json
    ) AS json_type,
    -- Array type (array of integers)
    ARRAY [1, 2, 3] AS array_int_type,
    -- Map type (map from varchar to int)
    MAP(
        ARRAY ['key1', 'key2'],
        ARRAY [1, 2]
    ) AS map_varchar_int_type,
    -- UUID type
    CAST(
        '123e4567-e89b-12d3-a456-426614174000' AS uuid
    ) AS uuid_type
