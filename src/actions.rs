use diesel::prelude::*;

use crate::models;

/// Run query using Diesel to find a page from it's id
pub fn find_page_by_id(
    page_id: i32,
    conn: &PgConnection,
) -> Result<Option<models::Page>, diesel::result::Error> {
    use crate::schema::pages::dsl::*;

    let page = pages
        .filter(id.eq(page_id))
        .first::<models::Page>(conn)
        .optional()?;

    Ok(page)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_page(
    // prevent collision with `name` column imported inside the function
    new_page: models::NewPage,
    conn: &PgConnection,
) -> Result<models::Page, diesel::result::Error> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::pages::dsl::*;

    let page = diesel::insert_into(pages)
        .values(&new_page)
        .get_result::<models::Page>(conn)?;

    Ok(page)
}
