use crate::domain::models::User;

/// User extracted from the access token and injected into GraphQL context.
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user: User,
}

