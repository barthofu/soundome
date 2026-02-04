#[macro_export]
macro_rules! delete_with_relations {
    ($conn:expr, $id:expr, [ $( ($table:expr, $col:expr, $msg:expr) ),+ $(,)? ]) => {
        // On utilise diesel::result::Error comme type de transaction pour satisfaire Diesel
        $conn.transaction::<_, diesel::result::Error, _>(|conn| {
            $(
                diesel::delete($table.filter($col.eq($id)))
                    .execute(conn)
                    .map_err(|e| {
                        tracing::error!(
                            concat!($msg, ": {}"),
                            e
                        );
                        e
                    })?; // ? send diesel::result::Error
            )+
            Ok(())
        })
        .map_err(|err| {
            shared::errors::Error::Database(format!("Transaction failed: {}", err))
        })
    };
}
