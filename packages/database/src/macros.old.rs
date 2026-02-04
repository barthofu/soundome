pub mod resource {

    // get by id macro

    macro_rules! find_one {
        ($table: ident, $type: ident) => {
            pub fn find_one(conn: &mut SqliteConnection, id: i32) -> shared::types::SoundomeResult<$type> {
                crate::schema::$table::table.find(id).first::<$type>(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to get resource by id: {}",
                            err
                        ))
                    })
            }
        };
    }

    macro_rules! find_one_method {
        ($table: ident, $type: ident) => {
            fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> shared::types::SoundomeResult<$type> {
                crate::schema::$table::table.find(id).first::<$type>(self.conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to get resource by id: {}",
                            err
                        ))
                    })
            }
        };
    }

    // get all macro

    macro_rules! find_all {
        ($table: ident, $type: ident) => {
            pub fn find_all(conn: &mut SqliteConnection) -> shared::types::SoundomeResult<Vec<$type>> {
                crate::schema::$table::table
                    .select(crate::schema::$table::all_columns)
                    .load::<$type>(conn)
                    .map(|mut resources| {
                        resources.sort();
                        resources
                    })
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to get all resources: {}",
                            err
                        ))
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
            ) -> shared::types::SoundomeResult<$type> {
                diesel::insert_into(crate::schema::$table::table)
                    .values(&new_resource)
                    .execute(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to create resource: {}",
                            err
                        ))
                    })?;

                crate::schema::$table::table
                    .order(crate::schema::$table::id.desc())
                    .first::<$type>(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to create resource: {}",
                            err
                        ))
                    })
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
            ) -> shared::types::SoundomeResult<$type> {
                diesel::update(crate::schema::$table::table.find(id))
                    .set(&update_data)
                    .execute(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to update resource: {}",
                            err
                        ))
                    })?;

                crate::schema::$table::table.find(id).first::<$type>(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to update resource: {}",
                            err
                        ))
                    })
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
    pub(crate) use find_one_method;
    pub(crate) use update;
}

pub mod association {

    pub mod many_to_many {

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
                ) -> shared::types::SoundomeResult<Vec<$second_resource_type>> {
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
                        .map_err(|err| {
                            shared::errors::Error::Database(format!(
                                "Failed to get all associations: {}",
                                err
                            ))
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
                $first_foreign_key: ident,
                $second_foreign_key: ident,
            ) => {
                pub fn $function_name(
                    conn: &mut SqliteConnection,
                    first_resource: &$first_resource_type,
                    second_resource: &$second_resource_type,
                ) -> shared::types::SoundomeResult<$association_type> {
                    diesel::insert_into(crate::schema::$association_table::table)
                        .values(&$association_type {
                            $first_foreign_key: first_resource.id,
                            $second_foreign_key: second_resource.id,
                        })
                        .execute(conn)
                        .map_err(|err| {
                            shared::errors::Error::Database(format!(
                                "Failed to create association: {}",
                                err
                            ))
                        })?;

                    crate::schema::$association_table::table
                        .order(crate::schema::$association_table::$first_foreign_key.desc())
                        .first::<$association_type>(conn)
                        .map_err(|err| {
                            shared::errors::Error::Database(format!(
                                "Failed to create association: {}",
                                err
                            ))
                        })
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

    // pub mod one_to_many {
    //     macro_rules! get_association {
    //         (
    //             $function_name: ident,
    //             $first_resource_type: ident,
    //             $second_resource_table: ident,
    //             $second_resource_type: ident,
    //             $association_table: ident,
    //             $association_type: ident,
    //             $association_foreign_key: ident,
    //         ) => {
    //             pub fn $function_name(
    //                 conn: &mut SqliteConnection,
    //                 resource: &$first_resource_type,
    //             ) -> diesel::QueryResult<Vec<$second_resource_type>> {
    //                 crate::schema::$second_resource_table::table
    //                     .select(crate::schema::$second_resource_table::all_columns)
    //                     .filter(
    //                         crate::schema::$second_resource_table::$association_foreign_key
    //                             .eq(resource.id),
    //                     )
    //                     .load::<$second_resource_type>(conn)
    //                     .map(|mut resources| {
    //                         resources.sort();
    //                         resources
    //                     })
    //             }
    //         };
    //     }
    // }

}
