pub mod resource {

    // get by id macro

    macro_rules! find_one {
        ($table: ident, $type: ident) => {
            pub fn find_one(conn: &mut SqliteConnection, id: i32) -> QueryResult<$type> {
                crate::schema::$table::table.find(id).first::<$type>(conn)
            }
        };
    }

    // get all macro

    macro_rules! find_all {
        ($table: ident, $type: ident) => {
            pub fn find_all(conn: &mut SqliteConnection) -> QueryResult<Vec<$type>> {
                crate::schema::$table::table
                    .select(crate::schema::$table::all_columns)
                    .load::<$type>(conn)
                    .map(|mut resources| {
                        resources.sort();
                        resources
                    })
            }
        };
    }

    // create macro

    macro_rules! create {
        ($table: ident, $type: ident, $new_type: ident) => {
            pub fn create(
                conn: &mut SqliteConnection,
                new_resource: $new_type,
            ) -> QueryResult<$type> {
                diesel::insert_into(crate::schema::$table::table)
                    .values(&new_resource)
                    .execute(conn)?;

                crate::schema::$table::table
                    .order(crate::schema::$table::id.desc())
                    .first::<$type>(conn)
            }
        };
    }

    // update macro

    macro_rules! update {
        ($table: ident, $type: ident, $update_type: ident) => {
            pub fn update(
                conn: &mut SqliteConnection,
                id: i32,
                update_data: $update_type,
            ) -> QueryResult<$type> {
                diesel::update(crate::schema::$table::table.find(id))
                    .set(&update_data)
                    .execute(conn)?;

                crate::schema::$table::table.find(id).first::<$type>(conn)
            }
        };
    }

    // delete macro

    macro_rules! delete {
        ($table: ident) => {
            pub fn delete(conn: &mut SqliteConnection, id: i32) -> bool {
                diesel::delete(crate::schema::$table::table.find(id))
                    .execute(conn)
                    .is_ok()
            }
        };
    }

    pub(crate) use create;
    pub(crate) use delete;
    pub(crate) use find_all;
    pub(crate) use find_one;
    pub(crate) use update;
}

pub mod association {

    macro_rules! get_all_associations {
        (
            $function_name: ident,
            $first_resource_type: ident,
            $second_resource_table: ident,
            $second_resource_type: ident,
            $association_table: ident,
            $association_type: ident,
            $association_foreign_key: ident,
        ) => {
            pub fn $function_name(
                conn: &mut SqliteConnection,
                resource: &$first_resource_type,
            ) -> QueryResult<Vec<$second_resource_type>> {
                let ids = $association_type::belonging_to(resource)
                    .select(crate::schema::$association_table::$association_foreign_key);

                crate::schema::$second_resource_table::table
                    .select(crate::schema::$second_resource_table::all_columns)
                    .filter(crate::schema::$second_resource_table::id.eq_any(ids))
                    .load::<$second_resource_type>(conn)
                    .map(|mut resources| {
                        resources.sort();
                        resources
                    })
            }
        };
    }

    // macro_rules! get_association {

    //     (
    //         $function_name: ident,
    //         $first_resource_type: ident,
    //         $second_resource_table: ident,
    //         $second_resource_type: ident,
    //         $association_table: ident,
    //         $association_type: ident,
    //         $association_foreign_key: ident,
    //     ) => {
    //         pub fn $function_name(resource: &$first_resource_type, id: i32, conn: &mut SqliteConnection) -> QueryResult<$second_resource_type> {

    //             let association = $association_type::belonging_to(resource)
    //                 .select(crate::schema::$association_table::$association_foreign_key)
    //                 .filter(crate::schema::$association_table::$association_foreign_key.eq(id))
    //                 .first::<i32>(conn)?;

    //             if (association == id) {
    //                 crate::schema::$second_resource_table::table
    //                     .find(id)
    //                     .first::<$second_resource_type>(conn)
    //             } else {
    //                 Err(diesel::result::Error::NotFound)
    //             }

    //             crate::schema::$second_resource_table::table
    //                 .select(crate::schema::$second_resource_table::all_columns)
    //                 .filter(crate::schema::$second_resource_table::id.eq_any(ids))
    //                 .load::<$second_resource_type>(conn)
    //                 .map(|mut resources| {
    //                     resources.sort();
    //                     resources
    //                 })
    //         }
    //     }

    // }

    macro_rules! create_association {
        (
            $function_name: ident,
            $first_resource_type: ident,
            $second_resource_type: ident,
            $association_table: ident,
            $association_type: ident,
            $new_association_type: ident,
            $first_foreign_key: ident,
            $second_foreign_key: ident,
        ) => {
            pub fn $function_name(
                conn: &mut SqliteConnection,
                first_resource: &$first_resource_type,
                second_resource: &$second_resource_type,
            ) -> QueryResult<$association_type> {
                diesel::insert_into(crate::schema::$association_table::table)
                    .values(&$new_association_type {
                        $first_foreign_key: first_resource.id,
                        $second_foreign_key: second_resource.id,
                    })
                    .execute(conn)?;

                crate::schema::$association_table::table
                    .order(crate::schema::$association_table::id.desc())
                    .first::<$association_type>(conn)

                // let association = $association_type::belonging_to(first_resource)
                //     .select(crate::schema::$association_table::$second_foreign_key)
                //     .filter(crate::schema::$association_table::$second_foreign_key.eq(second_resource.id))
                //     .first::<i32>(conn);

                // match association.is_ok() {

                //     true => Err(crate::utils::error::Error::AlreadyExists),
                //     false => {

                //         diesel::insert_into(crate::schema::$association_table::table)
                //             .values(&$new_association_type {
                //                 $first_foreign_key: first_resource.id,
                //                 $second_foreign_key: second_resource.id,
                //             })
                //             .execute(conn)?;

                //         Ok(crate::schema::$association_table::table
                //             .order(crate::schema::$association_table::id.desc())
                //             .first::<$association_type>(conn)
                //         )
                //     }
                // }
            }
        };
    }

    macro_rules! delete_association {
        (
            $function_name: ident,
            $first_resource_type: ident,
            $second_resource_type: ident,
            $association_table: ident,
            $first_foreign_key: ident,
            $second_foreign_key: ident,
        ) => {
            pub fn $function_name(
                conn: &mut SqliteConnection,
                first_resource: &$first_resource_type,
                second_resource: &$second_resource_type,
            ) -> bool {
                diesel::delete(
                    crate::schema::$association_table::table
                        .filter(
                            crate::schema::$association_table::$first_foreign_key
                                .eq(first_resource.id),
                        )
                        .filter(
                            crate::schema::$association_table::$second_foreign_key
                                .eq(second_resource.id),
                        ),
                )
                .execute(conn)
                .is_ok()
            }
        };
    }

    pub(crate) use create_association;
    pub(crate) use delete_association;
    pub(crate) use get_all_associations;
}
