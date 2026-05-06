pub mod resource {

    macro_rules! get {
        ($route: literal, $arg_name: ident, $database: ident, $type: ident) => {
            #[openapi]
            #[get($route)]
            pub async fn get(
                $arg_name: i32,
                db: Db,
            ) -> Result<Json<$type>, crate::utils::error::Error> {
                db.run(move |c| crate::database::$database::find_one(c, $arg_name))
                    .await
                    .map(Json)
                    .map_err(|_| crate::utils::error::Error::NotFound)
            }
        };
    }

    macro_rules! get_all {
        ($route: literal, $database: ident, $type: ident) => {
            #[openapi]
            #[get($route)]
            pub async fn get_all(db: Db) -> Result<Json<Vec<$type>>, crate::utils::error::Error> {
                db.run(|c| crate::database::$database::find_all(c))
                    .await
                    .map(Json)
                    .map_err(|_| crate::utils::error::Error::Internal)
            }
        };
    }

    macro_rules! create {
        ($route: literal, $database: ident, $type: ident, $new_type: ident) => {
            #[openapi]
            #[post($route, format = "application/json", data = "<new_data>")]
            pub async fn create(
                new_data: Json<$new_type>,
                db: Db,
            ) -> Result<Json<$type>, crate::utils::error::Error> {
                db.run(|c| crate::database::$database::create(c, new_data.into_inner()))
                    .await
                    .map(Json)
                    .map_err(|_| crate::utils::error::Error::Internal)
            }
        };
    }

    macro_rules! update {
        ($route: literal, $arg_name: ident, $database: ident, $type: ident, $update_type: ident) => {
            #[openapi]
            #[patch($route, format = "application/json", data = "<update_data>")]
            pub async fn update(
                $arg_name: i32,
                update_data: Json<$update_type>,
                db: Db,
            ) -> Result<Json<$type>, crate::utils::error::Error> {
                db.run(move |c| {
                    crate::database::$database::update(c, $arg_name, update_data.into_inner())
                })
                .await
                .map(Json)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => crate::utils::error::Error::NotFound,
                    _ => crate::utils::error::Error::NotModified,
                })
            }
        };
    }

    macro_rules! delete {
        ($route: literal, $arg_name: ident, $database: ident) => {
            #[openapi]
            #[delete($route)]
            pub async fn delete($arg_name: i32, db: Db) -> Json<response::Success> {
                let success = db
                    .run(move |c| crate::database::$database::delete(c, $arg_name))
                    .await;
                Json(response::Success { success })
            }
        };
    }
}

pub mod association {

    macro_rules! get {
        (
            $function_name: ident,
            $route: literal,
            $arg_name: ident,
            $database: ident,
            $type: ident
        ) => {
            #[openapi]
            #[get($route)]
            pub async fn $function_name(
                $arg_name: i32,
                db: Db,
            ) -> Result<Json<Vec<$type>>, crate::utils::error::Error> {
                db.run(move |c| {
                    let resource = crate::database::$database::find_one(c, $arg_name);
                    crate::database::$database::get_tracks(c, &resource.unwrap())
                })
                .await
                .map(Json)
                .map_err(|_| crate::utils::error::Error::NotFound)
            }
        };
    }

    macro_rules! associate {
        (
            $function_name: ident,
            $route: literal,
            $first_arg_name: ident,
            $second_arg_name: ident,
            $first_database: ident,
            $second_database: ident,
            $type: ident
        ) => {
            #[openapi]
            #[post($route)]
            pub async fn $function_name(
                $first_arg_name: i32,
                $second_arg_name: i32,
                db: Db,
            ) -> Result<Json<$type>, crate::utils::error::Error> {
                db.run(move |c| {
                    let first_resource =
                        crate::database::$first_database::find_one(c, $first_arg_name)
                            .map_err(|_| crate::utils::error::Error::NotFound)?;

                    let second_resource =
                        crate::database::$second_database::find_one(c, $second_arg_name)
                            .map_err(|_| crate::utils::error::Error::NotFound)?;

                    let association_exists = crate::database::$first_database::has_track(
                        c,
                        &first_resource,
                        &second_resource,
                    );

                    if association_exists {
                        crate::database::$first_database::create_track(
                            c,
                            &first_resource,
                            &second_resource,
                        )
                        .map_err(|_| crate::utils::error::Error::Internal)
                    } else {
                        Err(crate::utils::error::Error::AlreadyExists)
                    }
                })
                .await
                .map(Json)
                .map_err(|e| match e {
                    crate::utils::error::Error::DieselError(_) => {
                        crate::utils::error::Error::NotFound
                    }
                    crate::utils::error::Error::AlreadyExists => {
                        crate::utils::error::Error::AlreadyExists
                    }
                    _ => crate::utils::error::Error::Internal,
                })
            }
        };
    }

    macro_rules! dissociate {
        (
            $function_name: ident,
            $route: literal,
            $first_arg_name: ident,
            $second_arg_name: ident,
            $first_database: ident,
            $second_database: ident
        ) => {
            #[openapi]
            #[delete($route)]
            pub async fn $function_name(
                $first_arg_name: i32,
                $second_arg_name: i32,
                db: Db,
            ) -> Json<response::Success> {
                let success = db
                    .run(move |c| {
                        let first_resource =
                            crate::database::$first_database::find_one(c, $first_arg_name);
                        let second_resource =
                            crate::database::$second_database::find_one(c, $second_arg_name);
                        crate::database::$first_database::delete_track(
                            c,
                            &first_resource.unwrap(),
                            &second_resource.unwrap(),
                        )
                    })
                    .await;

                Json(response::Success { success })
            }
        };
    }
}
