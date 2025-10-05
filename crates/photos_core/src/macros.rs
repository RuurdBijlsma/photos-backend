#[macro_export]
macro_rules! insert_query {
    (
        $tx:expr,
        $table:literal,
        { $($col:ident: $val:expr),+ $(,)? }
    ) => {
        async {
            let columns = stringify!($($col),+);
            let placeholders = (1..=$crate::count_tts!($($col)+))
                .map(|i| format!("${}", i))
                .collect::<Vec<_>>()
                .join(", ");

            let sql = format!("INSERT INTO {} ({}) VALUES ({})", $table, columns, placeholders);

            sqlx::query(&sql)
            $(
                .bind($val)
            )+
            .execute(&mut **$tx)
            .await
        }
    };
}

/// Helper macro to count the number of tokens.
#[macro_export]
macro_rules! count_tts {
    ($($tts:tt)*) => {<[()]>::len(&[$($crate::replace_expr!($tts ())),*])};
}

/// Helper macro for count_tts.
#[macro_export]
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}